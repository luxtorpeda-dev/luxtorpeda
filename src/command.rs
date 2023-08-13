extern crate hex;
extern crate json;
extern crate reqwest;

use iso9660::{DirectoryEntry, ISO9660Reader, ISODirectory, ISO9660};
use regex::Regex;
use std::env;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Read;
use std::io::{Error, ErrorKind};
use std::path::Path;
use std::process::Command;
use rbchunk;

use crate::client;
use crate::config;
use crate::package;
use crate::package::place_state_file;
use crate::package_metadata;
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
static LUX_STEAM_CLOUD: &str = "LUX_STEAM_CLOUD";

pub fn usage() {
    println!("usage: lux [run | wait-before-run | manual-download] <exe | app_id> [<exe_args>]");
}

pub fn find_game_command(
    info: &package_metadata::Game,
    args: &[&str],
) -> Option<(String, Vec<String>)> {
    let orig_cmd = args.join(" ");

    if let Some(command) = &info.command {
        return Some((command.to_string(), info.command_args.clone()));
    }

    if let Some(cmds) = &info.commands {
        for new_cmd in cmds {
            let re = Regex::new(&new_cmd.command_name).unwrap();
            if re.is_match(&orig_cmd) {
                return Some((new_cmd.cmd.clone(), new_cmd.args.clone()));
            }
        }
    }

    None
}

pub fn process_setup_details(
    setup_info: &package_metadata::Setup,
) -> io::Result<Vec<client::PromptRequestData>> {
    let mut setup_items = Vec::new();

    if let Some(license_path) = &setup_info.license_path {
        if Path::new(&license_path).exists() {
            let mut file = File::open(license_path)?;
            let mut file_buf = vec![];
            file.read_to_end(&mut file_buf)?;
            let file_str = String::from_utf8_lossy(&file_buf);
            let file_str_milk = file_str.as_ref();

            let prompt_request = client::PromptRequestData {
                label: Some("By clicking Ok below, you are agreeing to the following.".to_string()),
                prompt_type: "question".to_string(),
                title: "Closed Source Engine EULA".to_string(),
                prompt_id: "closedsourceengineeulaconfirm".to_string(),
                rich_text: Some(file_str_milk.to_string()),
            };
            setup_items.push(prompt_request);
        }
    }

    if let Some(dialogs) = &setup_info.dialogs {
        for entry in dialogs {
            if entry.dialog_type == "input" {
                let prompt_request = client::PromptRequestData {
                    label: Some(entry.label.to_string()),
                    prompt_type: "input".to_string(),
                    title: entry.title.to_string(),
                    prompt_id: std::format!("dialogentryconfirm%%{}%%", entry.key).to_string(),
                    rich_text: None,
                };
                setup_items.push(prompt_request);
            }
        }
    }

    Ok(setup_items)
}

fn run_bchunk(bchunk_info: &package_metadata::SetupBChunk) -> io::Result<()> {
    // TODO: process generate_cue_file if exists; clean up tomb raider script to not run these steps anymore, make sure that is run via ci
    let args = rbchunk::Args {
        bin_file: bchunk_info.bin_file.to_string(),
        cue_file: bchunk_info.cue_file.to_string(),
        verbose: true,
        output_name: "".to_string(),
        psx_truncate: false,
        raw: false,
        swap_audo_bytes: false,
        to_wav: false
    };
    match rbchunk::convert(args) {
        Ok(()) => {
            info!("run_bchunk, Conversion complete!");
            Ok(())
        },
        Err(err) => {
            error!("run_bchunk failed {}", err);
            return Err(Error::new(
                ErrorKind::Other,
                format!("run_bchunk failed - {}", err),
            ));
        }
    }
}

fn iso_extract_tree<T: ISO9660Reader>(dir: &ISODirectory<T>, level: u32) -> io::Result<()> {
    // TODO: change this to log info! of full path and extract (based on if this matches the /DATA sort of prefix of path in config)
    for entry_item in dir.contents() {
        match entry_item {
            Ok(entry) => match entry {
                DirectoryEntry::Directory(dir) => {
                    if dir.identifier == "." || dir.identifier == ".." {
                        continue;
                    }
                    for _i in 0..level {
                        print!("  ");
                    }
                    println!("- {}/", dir.identifier);
                    return iso_extract_tree(&dir, level + 1);
                }
                DirectoryEntry::File(file) => {
                    for _i in 0..level {
                        print!("  ");
                    }
                    println!("- {}", file.identifier);
                }
            },
            Err(err) => {
                error!("iso_extract_tree err: {:?}", err);
                return Err(Error::new(ErrorKind::Other, "iso_extract_tree failed"));
            }
        }
    }

    Ok(())
}

fn run_iso_extract(iso_extract_info: &package_metadata::SetupIsoExtract) -> io::Result<()> {
    //TODO: make file_path optional and do a file path search (file path search if nothing found should not error out, just continue on)
    match std::fs::File::open(&iso_extract_info.file_path) {
        Ok(file) => match ISO9660::new(file) {
            Ok(iso) => iso_extract_tree(&iso.root, 0),
            Err(err) => {
                error!("run_iso_extract iso read err: {:?}", err);
                return Err(Error::new(
                    ErrorKind::Other,
                    "run_iso_extract failed, iso read error",
                ));
            }
        },
        Err(err) => {
            error!("run_iso_extract file open err: {:?}", err);
            return Err(Error::new(
                ErrorKind::Other,
                "run_iso_extract failed, file open error",
            ));
        }
    }
}

pub fn run_setup(
    setup_info: &package_metadata::Setup,
    sender: &std::sync::mpsc::Sender<String>,
) -> io::Result<()> {
    let command_str = setup_info.command.to_string();
    info!("setup run: \"{}\"", command_str);

    let status_obj = client::StatusObj {
        log_line: Some(format!("setup run: \"{}\"", command_str)),
        ..Default::default()
    };
    let status_str = serde_json::to_string(&status_obj).unwrap();
    sender.send(status_str).unwrap();

    if let Some(bchunk_info) = &setup_info.bchunk {
        info!("setup run bchunk");
        let status_obj = client::StatusObj {
            log_line: Some(format!("setup running bchunk")),
            ..Default::default()
        };
        let status_str = serde_json::to_string(&status_obj).unwrap();
        sender.send(status_str).unwrap();

        match run_bchunk(&bchunk_info) {
            Ok(()) => {}
            Err(err) => {
                error!("command::run_bchunk err: {:?}", err);
                return Err(err);
            }
        }
    }

    if let Some(iso_extract_info) = &setup_info.iso_extract {
        info!("setup run iso_extract");
        match run_iso_extract(&iso_extract_info) {
            Ok(()) => {}
            Err(err) => {
                error!("command::run_iso_extract err: {:?}", err);
                return Err(err);
            }
        }
    }

    let setup_cmd = Command::new(command_str)
        .env("LD_PRELOAD", "")
        .status()
        .expect("failed to execute process");

    if !setup_cmd.success() {
        return Err(Error::new(ErrorKind::Other, "setup failed"));
    }

    File::create(setup_info.complete_path.clone())?;

    Ok(())
}

pub fn run(
    args: &[&str],
    engine_choice: String,
    sender: &std::sync::mpsc::Sender<String>,
    after_setup_question_mode: bool,
) -> io::Result<package_metadata::Game> {
    env::set_var(LUX_ERRORS_SUPPORTED, "1");

    let app_id = user_env::steam_app_id();

    let mut game_info = match package::get_game_info(&app_id) {
        Ok(game_info) => game_info,
        Err(err) => {
            return Err(err);
        }
    };

    if game_info.choices.is_some() {
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
    info!("{:?}", game_info);

    if game_info.use_original_command_directory {
        let tmp_path = Path::new(args[0]);
        let parent_path = tmp_path.parent().unwrap();
        env::set_current_dir(parent_path).unwrap();

        info!("original command: {:?}", args);
        info!("working dir: {:?}", env::current_dir());
        info!("tool dir: {:?}", user_env::tool_dir());
    }

    if !after_setup_question_mode {
        match package::install(&game_info, sender) {
            Ok(()) => {}
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
    game_info: &package_metadata::Game,
    sender: &std::sync::mpsc::Sender<String>,
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

    match find_game_command(game_info, args) {
        None => ret = Err(Error::new(ErrorKind::Other, "No command line defined")),
        Some((cmd, cmd_args)) => {
            info!("run: \"{}\" with args: {:?} {:?}", cmd, cmd_args, exe_args);

            let status_obj = client::StatusObj {
                log_line: Some(format!(
                    "run: \"{}\" with args: {:?} {:?}",
                    cmd, cmd_args, exe_args
                )),
                ..Default::default()
            };
            let status_str = serde_json::to_string(&status_obj).unwrap();
            sender.send(status_str).unwrap();

            match Command::new(cmd)
                .args(cmd_args)
                .args(exe_args)
                .env(LUX_ORIGINAL_EXE, args[0])
                .env(LUX_ORIGINAL_EXE_FILE, exe_file)
                .spawn()
            {
                Ok(mut child) => {
                    let config = config::Config::from_config_file();
                    if config.close_client_on_launch {
                        info!("closing client without waiting on engine close");
                        std::process::exit(0);
                    }
                    match child.wait() {
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
                Err(err) => {
                    ret = Err(err);
                }
            }
        }
    };

    if ret.is_ok() {
        std::process::exit(0);
    } else {
        ret
    }
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

    let config = config::Config::from_config_file();
    if config.enable_steam_cloud {
        info!("enable_steam_cloud");
        env::set_var(LUX_STEAM_CLOUD, "1");
    }

    Ok(())
}
