extern crate hex;
extern crate json;
extern crate reqwest;

use regex::Regex;
use std::env;
use std::fs;
use std::fs::File;
use std::io;
use std::io::{Error, ErrorKind};
use std::path::Path;
use std::process::Command;

use crate::client;
use crate::package;
use crate::package::place_state_file;
use crate::user_env;

extern crate log;
extern crate simplelog;
use log::{debug, error, info};
use simplelog::*;

static ORIGINAL_LD_PRELOAD: &str = "ORIGINAL_LD_PRELOAD";
static LD_PRELOAD: &str = "LD_PRELOAD";

static STEAM_DECK_ENV: &str = "SteamDeck";
static STEAM_OS_ENV: &str = "SteamOS";
static USER_ENV: &str = "USER";
static STEAM_DECK_USER: &str = "deck";

static LUX_ERRORS_SUPPORTED: &str = "LUX_ERRORS_SUPPORTED";
static LUX_ORIGINAL_EXE: &str = "LUX_ORIGINAL_EXE";
static LUX_ORIGINAL_EXE_FILE: &str = "LUX_ORIGINAL_EXE_FILE";
static LUX_WRITE_LOGGING: &str = "LUX_WRITE_LOGGING";
static LUX_STEAM_DECK: &str = "LUX_STEAM_DECK";
static LUX_STEAM_DECK_GAMING_MODE: &str = "LUX_STEAM_DECK_GAMING_MODE";

fn usage() {
    println!("usage: lux [run | wait-before-run | manual-download] <exe | app_id> [<exe_args>]");
}

fn json_to_args(args: &json::JsonValue) -> Vec<String> {
    args.members()
        .map(|j| j.as_str())
        .skip_while(|o| o.is_none()) // filter?
        .map(|j| j.unwrap().to_string())
        .collect()
}

fn find_game_command(info: &json::JsonValue, args: &[&str]) -> Option<(String, Vec<String>)> {
    let orig_cmd = args.join(" ");

    if !info["command"].is_null() {
        let new_prog = info["command"].to_string();
        let new_args = json_to_args(&info["command_args"]);
        return Some((new_prog, new_args));
    }

    if info["commands"].is_null() {
        return None;
    }

    let cmds = &info["commands"];
    for (expr, new_cmd) in cmds.entries() {
        let re = Regex::new(expr).unwrap();
        if re.is_match(&orig_cmd) {
            let new_prog = new_cmd["cmd"].to_string();
            let new_args = json_to_args(&new_cmd["args"]);
            return Some((new_prog, new_args));
        }
    }

    None
}

fn run_setup(game_info: &json::JsonValue) -> io::Result<()> {
    let setup_info = &game_info["setup"];
    if !package::is_setup_complete(&game_info["setup"]) {
        if !&setup_info["license_path"].is_null()
            && Path::new(&setup_info["license_path"].to_string()).exists()
        {
            /*match dialog::show_file_with_confirm(
                "Closed Source Engine EULA",
                &setup_info["license_path"].to_string(),
            ) {
                Ok(()) => {
                    info!("show eula. dialog was accepted");
                }
                Err(_) => {
                    info!("show eula. dialog was rejected");
                    if !setup_info["uninstall_command"].is_null() {
                        let command_str = setup_info["uninstall_command"].to_string();
                        info!("uninstall run: \"{}\"", command_str);

                        Command::new(command_str)
                            .env("LD_PRELOAD", "")
                            .status()
                            .expect("failed to execute process");
                    }
                    return Err(Error::new(ErrorKind::Other, "dialog was rejected"));
                }
            }*/
        }

        if !&setup_info["dialogs"].is_null() {
            for entry in setup_info["dialogs"].members() {
                if entry["type"] == "input" {
                    /*match dialog::text_input(
                        &entry["title"].to_string(),
                        &entry["label"].to_string(),
                        &entry["key"].to_string(),
                    ) {
                        Ok(_) => {}
                        Err(err) => {
                            error!("setup failed, text input dialog error: {:?}", err);
                            return Err(Error::new(
                                ErrorKind::Other,
                                "setup failed, input dialog failed",
                            ));
                        }
                    };*/
                }
            }
        }

        let command_str = setup_info["command"].to_string();
        info!("setup run: \"{}\"", command_str);
        let setup_cmd = Command::new(command_str)
            .env("LD_PRELOAD", "")
            .status()
            .expect("failed to execute process");

        if !setup_cmd.success() {
            return Err(Error::new(ErrorKind::Other, "setup failed"));
        }

        File::create(&setup_info["complete_path"].to_string())?;
    }

    Ok(())
}

pub fn run(
    args: &[&str],
    engine_choice: String,
    sender: &std::sync::mpsc::Sender<String>,
) -> io::Result<json::JsonValue> {
    env::set_var(LUX_ERRORS_SUPPORTED, "1");

    let app_id = user_env::steam_app_id();

    info!("luxtorpeda version: {}", env!("CARGO_PKG_VERSION"));
    info!("steam_app_id: {:?}", &app_id);
    info!("original command: {:?}", args);
    info!("working dir: {:?}", env::current_dir());
    info!("tool dir: {:?}", user_env::tool_dir());

    let mut game_info = match package::get_game_info(&app_id) {
        Ok(game_info) => game_info,
        Err(err) => {
            return Err(err);
        }
    };

    if game_info.is_null() {
        return Err(Error::new(ErrorKind::Other, "Unknown app_id"));
    }

    if !game_info["choices"].is_null() {
        match package::convert_game_info_with_choice(engine_choice, &mut game_info) {
            Ok(()) => {
                info!("engine choice complete");
            }
            Err(err) => {
                return Err(err);
            }
        };
    }

    info!("json:");
    info!("{:#}", game_info);

    if game_info["use_original_command_directory"] == true {
        let tmp_path = Path::new(args[0]);
        let parent_path = tmp_path.parent().unwrap();
        env::set_current_dir(parent_path).unwrap();

        info!("original command: {:?}", args);
        info!("working dir: {:?}", env::current_dir());
        info!("tool dir: {:?}", user_env::tool_dir());
    }

    if !game_info["download"].is_null() {
        package::install(&game_info, sender)?;
    }

    if !game_info["setup"].is_null() {
        match run_setup(&game_info) {
            Ok(()) => {
                info!("setup complete");
            }
            Err(err) => {
                return Err(err);
            }
        }
    }

    match env::var(ORIGINAL_LD_PRELOAD) {
        Ok(val) => {
            env::set_var(LD_PRELOAD, val);
        }
        Err(err) => {
            info!("ORIGINAL_LD_PRELOAD not found: {}", err);
        }
    }

    Ok(game_info)
}

pub fn run_wrapper(
    args: &[&str],
    engine_choice: String,
    sender: std::sync::mpsc::Sender<String>,
) -> io::Result<()> {
    if args.is_empty() {
        usage();
        std::process::exit(0)
    }

    let exe_args = &args[1..];

    let mut exe_file = "";
    let exe_path = Path::new(args[0]);
    if let Some(exe_file_name) = exe_path.file_name() {
        if let Some(exe_file_str) = exe_file_name.to_str() {
            exe_file = exe_file_str;
        }
    }

    let mut ret: Result<(), Error> = Ok(());
    let mut game_info = None;

    match run(args, engine_choice, &sender) {
        Ok(g) => {
            game_info = Some(g);
        }
        Err(err) => {
            ret = Err(err);
        }
    }

    if let Some(game_info) = game_info {
        match find_game_command(&game_info, args) {
            None => ret = Err(Error::new(ErrorKind::Other, "No command line defined")),
            Some((cmd, cmd_args)) => {
                info!("run: \"{}\" with args: {:?} {:?}", cmd, cmd_args, exe_args);

                let status_obj = client::StatusObj {
                    label: None,
                    progress: None,
                    complete: false,
                    log_line: Some(format!(
                        "run: \"{}\" with args: {:?} {:?}",
                        cmd, cmd_args, exe_args
                    )),
                    error: None,
                };
                let status_str = serde_json::to_string(&status_obj).unwrap();
                sender.send(status_str).unwrap();

                match Command::new(cmd)
                    .args(cmd_args)
                    .args(exe_args)
                    .env(LUX_ORIGINAL_EXE, args[0])
                    .env(LUX_ORIGINAL_EXE_FILE, exe_file)
                    .status()
                {
                    Ok(status) => {
                        info!("run returned with {}", status);
                        if let Some(exit_code) = status.code() {
                            if exit_code == 10 {
                                info!("run returned with lux exit code");
                                match fs::read_to_string("last_error.txt") {
                                    Ok(s) => {
                                        ret = Err(Error::new(
                                            ErrorKind::Other,
                                            std::format!("Error on run: {}", s),
                                        ));
                                    }
                                    Err(err) => {
                                        error!("read err: {:?}", err);
                                    }
                                };
                            }
                        } else {
                            ret = Ok(());
                        }
                    }
                    Err(err) => {
                        ret = Err(err);
                    }
                };
            }
        };
    }

    if ret.is_ok() {
        std::process::exit(0);
    } else {
        ret
    }
}

fn manual_download(args: &[&str]) -> io::Result<()> {
    if args.is_empty() {
        usage();
        std::process::exit(0)
    }

    let app_id = args[0];
    package::update_packages_json().unwrap();
    //package::download_all(app_id.to_string())?;

    Ok(())
}

fn setup_logging(file: Option<File>) {
    if let Some(file) = file {
        match CombinedLogger::init(vec![
            TermLogger::new(
                LevelFilter::Info,
                Config::default(),
                TerminalMode::Mixed,
                ColorChoice::Auto,
            ),
            WriteLogger::new(LevelFilter::Info, Config::default(), file),
        ]) {
            Ok(()) => {
                info!("setup_logging with write success");
            }
            Err(err) => {
                println!("setup_logging with write error: {:?}", err);
            }
        }
    } else {
        match CombinedLogger::init(vec![TermLogger::new(
            LevelFilter::Info,
            Config::default(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        )]) {
            Ok(()) => {
                info!("setup_logging success");
            }
            Err(err) => {
                println!("setup_logging error: {:?}", err);
            }
        }
    }
}

pub fn main() -> io::Result<()> {
    let env_args: Vec<String> = env::args().collect();
    let args: Vec<&str> = env_args.iter().map(|a| a.as_str()).collect();

    if args.len() < 2 {
        usage();
        std::process::exit(0)
    }

    user_env::assure_xdg_runtime_dir()?;
    user_env::assure_tool_dir(args[0])?;

    match env::var(LUX_WRITE_LOGGING) {
        Ok(val) => {
            if val == "1" {
                match place_state_file("luxtorpeda.log") {
                    Ok(path) => {
                        println!("writing log to {:?}", path);
                        match File::create(path) {
                            Ok(file) => {
                                setup_logging(Some(file));
                            }
                            Err(err) => {
                                println!("log writeLogger create failure: {:?}", err);
                                setup_logging(None);
                            }
                        };
                    }
                    Err(_err) => {
                        setup_logging(None);
                    }
                };
            } else if val == "0" {
                setup_logging(None);
            }
        }
        Err(_err) => {
            setup_logging(None);
        }
    }

    let mut on_steam_deck = false;

    match env::var(STEAM_DECK_ENV) {
        Ok(val) => {
            if val == "1" {
                on_steam_deck = true;
            }
        }
        Err(err) => {
            debug!("SteamDeck env not found: {}", err);
        }
    }

    if !on_steam_deck {
        match env::var(USER_ENV) {
            Ok(val) => {
                if val == STEAM_DECK_USER {
                    on_steam_deck = true;
                }
            }
            Err(err) => {
                debug!("USER env not found: {}", err);
            }
        }
    }

    if on_steam_deck {
        info!("detected running on steam deck");
        env::set_var(LUX_STEAM_DECK, "1");

        match env::var(STEAM_OS_ENV) {
            Ok(val) => {
                if val == "1" {
                    info!("detected running on steam deck gaming mode");
                    env::set_var(LUX_STEAM_DECK_GAMING_MODE, "1");
                }
            }
            Err(err) => {
                debug!("STEAM_OS_ENV env not found: {}", err);
            }
        }
    }

    Ok(())
}
