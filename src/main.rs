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
use std::process::Command;
use sha1::{Sha1, Digest};
use std::path::Path;

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

pub fn get_remote_packages_hash(remote_hash_url: &String) -> Option<String> {
    let mut remote_hash_response = match reqwest::get(remote_hash_url) {
        Ok(s) => s,
        Err(err) => {
            println!("get_remote_packages_hash error in get: {:?}", err);
            return None;
        }
    };
    
    let remote_hash_str = match remote_hash_response.text() {
        Ok(s) => s,
        Err(err) => {
            println!("get_remote_packages_hash error in text: {:?}", err);
            return None;
        }
    };
    
    return Some(remote_hash_str);
}

pub fn generate_hash_from_file_path(file_path: &std::path::PathBuf) -> io::Result<String> {
    let json_str = fs::read_to_string(file_path)?;
    let mut hasher = Sha1::new();
    hasher.update(json_str);
    let hash_result = hasher.finalize();
    let hash_str = hex::encode(hash_result);
    return Ok(hash_str);
}

pub fn update_packages_json() -> io::Result<()> {
    let config_json_file = user_env::tool_dir().join("config.json");
    let config_json_str = fs::read_to_string(config_json_file)?;
    let config_parsed = json::parse(&config_json_str).unwrap();
    
    let should_do_update = &config_parsed["should_do_update"];
    if should_do_update != true {
        return Ok(());
    }
    
    let packages_json_file = user_env::tool_dir().join("packages.json");
    let mut should_download = true;
    let mut remote_hash_str: String = String::new();
    
    let remote_hash_url = std::format!("{0}/packages.hash", &config_parsed["host_url"]);
    match get_remote_packages_hash(&remote_hash_url) {
        Some(tmp_hash_str) => {
            remote_hash_str = tmp_hash_str;
        },
        None => {
            println!("update_packages_json in get_remote_packages_hash call. received none");
            should_download = false;
        }
    }
    
    if should_download {
        if !Path::new(&packages_json_file).exists() {
            should_download = true;
            println!("update_packages_json. packages.json does not exist");
        } else {
            let hash_str = generate_hash_from_file_path(&packages_json_file)?;
            println!("update_packages_json. found hash: {}", hash_str);
            
            println!("update_packages_json. found hash and remote hash: {0} {1}", hash_str, remote_hash_str);
            if hash_str != remote_hash_str {
                println!("update_packages_json. hash does not match. downloading");
                should_download = true;
            } else {
                should_download = false;
            }
        }
    }
    
    if should_download {
        println!("update_packages_json. downloading new packages.json");
        
        let remote_packages_url = std::format!("{0}/packages.json", &config_parsed["host_url"]);
        let mut download_complete = false;
        let local_packages_temp_path = user_env::tool_dir().join("packages-temp.json");
        
        match reqwest::get(&remote_packages_url) {
            Ok(mut response) => {
                let mut dest = fs::File::create(&local_packages_temp_path)?;
                io::copy(&mut response, &mut dest)?;
                download_complete = true;
            }
            Err(err) => {
                println!("update_packages_json. download err: {:?}", err);
            }
        }
        
        if download_complete {
            let new_hash_str = generate_hash_from_file_path(&local_packages_temp_path)?;
            if new_hash_str == remote_hash_str {
                println!("update_packages_json. new downloaded hash matches");
                fs::rename(local_packages_temp_path, packages_json_file)?;
            } else {
                println!("update_packages_json. new downloaded hash does not match");
                fs::remove_file(local_packages_temp_path)?;
            }
        }
    }
    
    Ok(())
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

    if !game_info["download"].is_null() {
        package::install()?;
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
    
    update_packages_json().unwrap();

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
