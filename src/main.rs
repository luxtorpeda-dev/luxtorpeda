extern crate json;
extern crate hex;
extern crate reqwest;

use regex::Regex;
use std::env;
use std::io;
use std::io::{Error, ErrorKind};
use std::path::Path;
use std::process::Command;
use std::fs::File;
use std::fs;

mod package;
mod pid_file;
mod user_env;
mod dialog;
mod mgmt;
mod ui;
mod run_context;

static SDL_VIRTUAL_GAMEPAD: &str = "SDL_GAMECONTROLLER_ALLOW_STEAM_VIRTUAL_GAMEPAD";
static SDL_IGNORE_DEVICES: &str = "SDL_GAMECONTROLLER_IGNORE_DEVICES";
static ORIGINAL_LD_PRELOAD: &str = "ORIGINAL_LD_PRELOAD";
static LD_PRELOAD: &str = "LD_PRELOAD";
static LUX_ERRORS_SUPPORTED: &str = "LUX_ERRORS_SUPPORTED";

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

fn run_setup(game_info: &json::JsonValue, context: Option<std::sync::Arc<std::sync::Mutex<run_context::RunContext>>>) -> io::Result<()> {
    let setup_info = &game_info["setup"];
    if !package::is_setup_complete(&game_info["setup"]) {
        if !&setup_info["license_path"].is_null() && Path::new(&setup_info["license_path"].to_string()).exists() {
            let license_context = context.clone();
            match dialog::show_file_with_confirm("Closed Source Engine EULA", &setup_info["license_path"].to_string(), license_context) {
                Ok(()) => {
                    println!("show eula. dialog was accepted");
                }
                Err(_) => {
                    println!("show eula. dialog was rejected");
                    if !setup_info["uninstall_command"].is_null() {
                        let command_str = setup_info["uninstall_command"].to_string();
                        println!("uninstall run: \"{}\"", command_str);

                        Command::new(command_str)
                            .env("LD_PRELOAD", "")
                            .status()
                            .expect("failed to execute process");
                    }
                    return Err(Error::new(ErrorKind::Other, "dialog was rejected"));
                }
            }
        }

        if !&setup_info["dialogs"].is_null() {
            for entry in setup_info["dialogs"].members() {
                let dialog_context = context.clone();
                if entry["type"] == "input" {
                    match dialog::text_input(&entry["title"].to_string(), &entry["label"].to_string(), &entry["key"].to_string(), dialog_context) {
                        Ok(_) => {},
                        Err(err) => {
                            println!("setup failed, text input dialog error: {:?}", err);
                            return Err(Error::new(ErrorKind::Other, "setup failed, input dialog failed"));
                        }
                    };
                }
            }
        }
                    
        let command_str = setup_info["command"].to_string();
        println!("setup run: \"{}\"", command_str);
        let setup_cmd = Command::new(command_str)
            .env("LD_PRELOAD", "")
            .status()
            .expect("failed to execute process");
            
        if !setup_cmd.success() {
            dialog::show_error(&"Setup Error".to_string(), &"Setup failed to complete".to_string(), context)?;
            return Err(Error::new(ErrorKind::Other, "setup failed"));
        }
                        
        File::create(&setup_info["complete_path"].to_string())?;
    }

    Ok(())
}

fn run(args: &[&str], context: Option<std::sync::Arc<std::sync::Mutex<run_context::RunContext>>>) -> io::Result<json::JsonValue> {
    let mut allow_virtual_gamepad = false;
    let mut ignore_devices = "".to_string();
    match env::var(SDL_VIRTUAL_GAMEPAD) {
        Ok(val) => {
            if val == "1" {
                 println!("turning virtual gamepad off");
                 env::remove_var(SDL_VIRTUAL_GAMEPAD);
                 allow_virtual_gamepad = true;

                 match env::var(SDL_IGNORE_DEVICES) {
                    Ok(val) => {
                        ignore_devices = val;
                        env::remove_var(SDL_IGNORE_DEVICES);
                    },
                    Err(err) => {
                         println!("SDL_IGNORE_DEVICES not found: {}", err);
                    }
                };
            }
        },
        Err(err) => {
            println!("virtual gamepad setting not found: {}", err);
        }
    }

    env::set_var(LUX_ERRORS_SUPPORTED, "1");

    package::update_packages_json().unwrap();

    let _pid_file = pid_file::new()?;
    let app_id = user_env::steam_app_id();

    println!("luxtorpeda version: {}", env!("CARGO_PKG_VERSION"));
    println!("steam_app_id: {:?}", &app_id);
    println!("original command: {:?}", args);
    println!("working dir: {:?}", env::current_dir());
    println!("tool dir: {:?}", user_env::tool_dir());
    
    let mut game_info = package::get_game_info(app_id.as_str(), context.clone())
        .ok_or_else(|| Error::new(ErrorKind::Other, "missing info about this game"))?;

    if game_info.is_null() {
        return Err(Error::new(ErrorKind::Other, "Unknown app_id"));
    }

    let download_context = context.clone();
    
    if !game_info["choices"].is_null() {
        let engine_choice = match package::download_all(app_id, download_context) {
            Ok(s) => s,
            Err(err) => {
                println!("download all error: {:?}", err);
                return Err(Error::new(ErrorKind::Other, "download all error"));
            }
        };
        match package::convert_game_info_with_choice(engine_choice, &mut game_info) {
            Ok(()) => {
                println!("engine choice complete");
            },
            Err(err) => {
                return Err(err);
            }
        };
    } else {
        package::download_all(app_id, download_context)?;
    }

    println!("json:");
    println!("{:#}", game_info);
    
    if game_info["use_original_command_directory"] == true {
        let tmp_path = Path::new(args[0]);
        let parent_path = tmp_path.parent().unwrap();
        env::set_current_dir(parent_path).unwrap();
        
        println!("original command: {:?}", args);
        println!("working dir: {:?}", env::current_dir());
        println!("tool dir: {:?}", user_env::tool_dir());
    }

    if !game_info["download"].is_null() {
        package::install(&game_info, context.clone())?;
    }
    
    if !game_info["setup"].is_null() {
        match run_setup(&game_info, context) {
            Ok(()) => {
                println!("setup complete");
            },
            Err(err) => {
                return Err(err);
            }
        }
    }

    if allow_virtual_gamepad {
        env::set_var(SDL_VIRTUAL_GAMEPAD, "1");
        env::set_var(SDL_IGNORE_DEVICES, ignore_devices);
    }

    match env::var(ORIGINAL_LD_PRELOAD) {
        Ok(val) => {
            env::set_var(LD_PRELOAD, val);
        },
        Err(err) => {
            println!("ORIGINAL_LD_PRELOAD not found: {}", err);
        }
    }

    Ok(game_info)
}

fn run_wrapper(args: &[&str]) -> io::Result<()> {
    if args.is_empty() {
        usage();
        std::process::exit(0)
    }

    let exe = args[0].to_lowercase();
    if exe.ends_with("iscriptevaluator.exe") {
        return Err(Error::new(ErrorKind::Other, "iscriptevaluator ignorning"));
    }

    let mut ret: Result<(), Error> = Ok(());
    let mut game_info = None;
    let (context, context_thread) = run_context::setup_run_context();
    let run_context = context.clone();

    match run(args, run_context) {
        Ok(g) => {
            game_info = Some(g);
        },
        Err(err) => {
            ret = Err(err);
        }
    }

    if let Some(close_context) = context {
        println!("sending close to run context thread");
        let mut guard = close_context.lock().unwrap();
        guard.thread_command = Some(run_context::ThreadCommand::Stop);
        std::mem::drop(guard);
    }

    context_thread.join().unwrap();

    if let Some(game_info) = game_info {
        let exe_args;
        if game_info["exe_in_args"] == true {
            exe_args = &args[0..];
        } else {
            exe_args = &args[1..];
        }

        match find_game_command(&game_info, args) {
            None => ret = Err(Error::new(ErrorKind::Other, "No command line defined")),
            Some((cmd, cmd_args)) => {
                println!("run: \"{}\" with args: {:?} {:?}", cmd, cmd_args, exe_args);
                match Command::new(cmd)
                    .args(cmd_args)
                    .args(exe_args)
                    .status() {
                        Ok(status) => {
                            println!("run returned with {}", status);
                            if let Some(exit_code) = status.code() {
                                if exit_code == 10 {
                                    println!("run returned with lux exit code");
                                    match fs::read_to_string("last_error.txt") {
                                        Ok(s) => {
                                            show_error_after_run("Run Error", &s)?;
                                        },
                                        Err(err) => {
                                            println!("read err: {:?}", err);
                                        }
                                    };
                                }
                            }
                            ret = Ok(());
                        },
                        Err(err) => {
                            ret = Err(err);
                        }
                    };
            }
        };
    }

    ret
}

fn show_error_after_run(title: &str, error_message: &str) -> io::Result<()> {
    let (context, context_thread) = run_context::setup_run_context();
    let close_context = context.clone();

    match env::var(SDL_VIRTUAL_GAMEPAD) {
        Ok(val) => {
            if val == "1" {
                 println!("turning virtual gamepad off");
                 env::remove_var(SDL_VIRTUAL_GAMEPAD);

                 match env::var(SDL_IGNORE_DEVICES) {
                    Ok(_val) => {
                        env::remove_var(SDL_IGNORE_DEVICES);
                    },
                    Err(err) => {
                         println!("SDL_IGNORE_DEVICES not found: {}", err);
                    }
                };
            }
        },
        Err(err) => {
            println!("virtual gamepad setting not found: {}", err);
        }
    };

    match dialog::show_error(title, error_message, context) {
        Ok(()) => {},
        Err(err) => {
            println!("error showing show_error: {:?}", err);
        }
    };

    if let Some(close_context) = close_context {
        println!("sending close to run context thread");
        let mut guard = close_context.lock().unwrap();
        guard.thread_command = Some(run_context::ThreadCommand::Stop);
        std::mem::drop(guard);
    }

    context_thread.join().unwrap();

    Ok(())
}

fn manual_download(args: &[&str]) -> io::Result<()> {
    if args.is_empty() {
        usage();
        std::process::exit(0)
    }
    
    let app_id = args[0];
    package::update_packages_json().unwrap();
    package::download_all(app_id.to_string(), None)?;
    
    Ok(())
}

fn main() -> io::Result<()> {
    let env_args: Vec<String> = env::args().collect();
    let args: Vec<&str> = env_args.iter().map(|a| a.as_str()).collect();

    if args.len() < 2 {
        usage();
        std::process::exit(0)
    }

    user_env::assure_xdg_runtime_dir()?;
    user_env::assure_tool_dir(args[0])?;

    let cmd = args[1];
    let cmd_args = &args[2..];

    match cmd {
        "run" => run_wrapper(cmd_args),
        "wait-before-run" => {
            pid_file::wait_while_exists();
            run_wrapper(cmd_args)
        },
        "waitforexitandrun" => {
            pid_file::wait_while_exists();
            run_wrapper(cmd_args)
        },
        "manual-download" => manual_download(cmd_args),
        "mgmt" => {
            package::update_packages_json().unwrap();
            mgmt::run_mgmt()
        },
        _ => {
            usage();
            std::process::exit(1)
        }
    }
}
