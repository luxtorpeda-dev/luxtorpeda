extern crate lazy_static;
extern crate json;
extern crate hex;
extern crate reqwest;

use regex::Regex;
use std::env;
use std::fs;
use std::io;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;
use std::path::Path;
use std::process::Command;
use std::fs::File;

mod package;
mod pid_file;
mod user_env;
mod dialog;

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

// crate glob might be useful here
fn find_metadata_json() -> io::Result<Vec<PathBuf>> {
    let files = fs::read_dir("manifests.lux")?
        .filter(|e| e.is_ok())
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().unwrap() == "json")
        .collect();
    Ok(files)
}

fn find_game_command(info: &json::JsonValue, args: &[&str]) -> Option<(String, Vec<String>)> {
    let orig_cmd = args.join(" ");

    // commands defined by packages

    for path in find_metadata_json().unwrap_or_default() {
        for repl in package::read_cmd_repl_from_file(path).unwrap_or_default() {
            if repl.match_cmd.is_match(&orig_cmd) {
                return Some((repl.cmd, repl.args));
            }
        }
    }

    // legacy commands from bundled packages.json file

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
        let re = Regex::new(expr).unwrap(); // TODO get rid of .unwrap
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
        if !&setup_info["license_path"].is_null() && Path::new(&setup_info["license_path"].to_string()).exists() {
            match dialog::show_file_with_confirm("Closed Source Engine EULA", &setup_info["license_path"].to_string()) {
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
                    
        let command_str = setup_info["command"].to_string();
        println!("setup run: \"{}\"", command_str);
        let setup_cmd = Command::new(command_str)
            .env("LD_PRELOAD", "")
            .status()
            .expect("failed to execute process");
            
        if !setup_cmd.success() {
            dialog::show_error(&"Setup Error".to_string(), &"Setup failed to complete".to_string())?;
            return Err(Error::new(ErrorKind::Other, "setup failed"));
        }
                        
        File::create(&setup_info["complete_path"].to_string())?;
        return Ok(());
    } else {
        return Ok(());
    }
}

fn run(args: &[&str], is_runtime: bool) -> io::Result<()> {
    if args.is_empty() {
        usage();
        std::process::exit(0)
    }

    let exe = args[0].to_lowercase();
    let exe_args = &args[1..];

    if exe.ends_with("iscriptevaluator.exe") {
        return Err(Error::new(ErrorKind::Other, "iscriptevaluator ignorning"));
    }

    package::update_packages_json(is_runtime).unwrap();

    let _pid_file = pid_file::new()?;
    let app_id = user_env::steam_app_id();

    println!("steam_app_id: {:?}", &app_id);
    println!("original command: {:?}", args);
    println!("working dir: {:?}", env::current_dir());
    println!("tool dir: {:?}", user_env::tool_dir());
    
    let mut game_info = package::get_game_info(app_id.as_str(), is_runtime)
        .ok_or_else(|| Error::new(ErrorKind::Other, "missing info about this game"))?;

    if game_info.is_null() {
        return Err(Error::new(ErrorKind::Other, "Unknown app_id"));
    }
    
    if !game_info["choices"].is_null() {
        let engine_choice = match package::download_all(app_id.to_string(), is_runtime) {
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
        package::download_all(app_id.to_string(), is_runtime)?;
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
        package::install(&game_info)?;
    }
    
    if !game_info["setup"].is_null() {
        match run_setup(&game_info) {
            Ok(()) => {
                println!("setup complete");
            },
            Err(err) => {
                return Err(err);
            }
        }
    }

    match find_game_command(&game_info, args) {
        None => Err(Error::new(ErrorKind::Other, "No command line defined")),
        Some((cmd, cmd_args)) => {
            println!("run: \"{}\" with args: {:?} {:?}", cmd, cmd_args, exe_args);
            Command::new(cmd)
                .args(cmd_args)
                .args(exe_args)
                .status()
                .expect("failed to execute process");
            Ok(())
        }
    }
}

fn manual_download(args: &[&str]) -> io::Result<()> {
    if args.is_empty() {
        usage();
        std::process::exit(0)
    }
    
    let app_id = args[0];
    package::update_packages_json(false).unwrap();
    package::download_all(app_id.to_string(), false)?;
    
    return Ok(());
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

    let mut cmd = args[1];
    let cmd_str = String::from(cmd);

    let cmd_args = &args[2..];
    let mut is_runtime = false;

    if cmd_str.contains("runtime_") {
        is_runtime = true;
        println!("run with runtime cmd: \"{}\"", cmd_str);

        let v: Vec<&str> = "Mary had a little lamb."
            .split("runtime_")
            .collect();
        cmd = v[0];
    }

    match cmd {
        "run" => run(cmd_args, is_runtime),
        "wait-before-run" => {
            pid_file::wait_while_exists();
            run(cmd_args, is_runtime)
        },
        "waitforexitandrun" => {
            pid_file::wait_while_exists();
            run(cmd_args, is_runtime)
        },
        "manual-download" => manual_download(cmd_args),
        _ => {
            usage();
            std::process::exit(1)
        }
    }
}
