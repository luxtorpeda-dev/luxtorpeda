#[macro_use]
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
use std::io::Read;
use std::io::Write;

mod fakescripteval;
mod ipc;
mod package;
mod pid_file;
mod user_env;

fn usage() {
    println!("usage: lux [run | wait-before-run] <exe> [<exe_args>]");
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
            let mut license_file = File::open(&setup_info["license_path"].to_string())?;
            let mut license_buf = vec![];
            license_file.read_to_end(&mut license_buf)?;
            let license_str = String::from_utf8_lossy(&license_buf);
            
            let mut converted_license_file = File::create("converted_license.txt")?;
            converted_license_file.write_all(license_str.as_bytes())?;
            
            let choice = Command::new("zenity")
                .args(&["--text-info", "--title=License", "--filename=converted_license.txt"])
                .status()
                .expect("failed to show eula");
                                    
            if !choice.success() {
                println!("show eula. dialog was rejected");
                
                if !setup_info["uninstall_command"].is_null() {
                    let command_str = setup_info["uninstall_command"].to_string();
                    println!("uninstall run: \"{}\"", command_str);
                    
                    Command::new(command_str)
                        .status()
                        .expect("failed to execute process");
                }
                
                return Err(Error::new(ErrorKind::Other, "dialog was rejected"));
            }
        }
                    
        let command_str = setup_info["command"].to_string();
        println!("setup run: \"{}\"", command_str);
        Command::new(command_str)
            .status()
            .expect("failed to execute process");
                        
        File::create(&setup_info["complete_path"].to_string())?;
        return Ok(());
    } else {
        return Ok(());
    }
}

fn run(args: &[&str]) -> io::Result<()> {
    if args.is_empty() {
        usage();
        std::process::exit(0)
    }

    let exe = args[0].to_lowercase();
    let exe_args = &args[1..];

    if exe.ends_with("iscriptevaluator.exe") {
        return fakescripteval::iscriptevaluator(exe_args);
    }
    
    package::update_packages_json().unwrap();

    let _pid_file = pid_file::new()?;
    let app_id = user_env::steam_app_id();

    println!("steam_app_id: {:?}", &app_id);
    println!("original command: {:?}", args);
    println!("working dir: {:?}", env::current_dir());
    println!("tool dir: {:?}", user_env::tool_dir());

    let packages_json_file = user_env::tool_dir().join("packages.json");
    let json_str = fs::read_to_string(packages_json_file)?;
    let parsed = json::parse(&json_str).unwrap();
    let game_info = &parsed[app_id];

    if game_info.is_null() {
        return Err(Error::new(ErrorKind::Other, "Unknown app_id"));
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
        package::install()?;
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

    match find_game_command(game_info, args) {
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
        "run" => run(cmd_args),
        "wait-before-run" => {
            pid_file::wait_while_exists();
            run(cmd_args)
        }
        _ => {
            usage();
            std::process::exit(1)
        }
    }
}
