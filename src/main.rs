use std::env;
use std::fs;
use std::io;
use std::io::{Error, ErrorKind};
use std::process::Command;

mod package;
mod pid_file;
mod user_env;

extern crate json;

fn usage() {
    println!("usage: lux [run | wait-before-run]");
}

fn run(args: &[String]) -> io::Result<()> {
    let _pid_file = pid_file::new()?;
    let app_id = user_env::steam_app_id();

    println!("working dir: {:?}", env::current_dir());
    println!("tool dir: {:?}", user_env::tool_dir());
    println!("args: {:?}", args);
    println!("steam_app_id: {:?}", &app_id);

    let packages_json_file = user_env::tool_dir().join("packages.json");
    let json_str = fs::read_to_string(packages_json_file)?;
    let parsed = json::parse(&json_str).unwrap();
    let game_info = &parsed[app_id];

    if game_info.is_null() {
        return Err(Error::new(ErrorKind::Other, "Unknown app_id"));
    }

    println!("json:");
    println!("{:#}", game_info);

    if !game_info["zipfile"].is_null() {
        let zip = game_info["zipfile"].to_string();
        package::install(zip)?;
    }

    if game_info["command"].is_null() {
        Err(Error::new(ErrorKind::Other, "No command line defined"))
    } else {
        let new_cmd = game_info["command"].to_string();
        Command::new(new_cmd)
            .status()
            .expect("failed to execute process");
        Ok(())
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        usage();
        std::process::exit(0)
    }

    let cmd = &args[1];
    let cmd_args = &args[2..];

    user_env::assure_xdg_runtime_dir()?;
    user_env::assure_tool_dir(&args[0])?;

    match cmd.as_str() {
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
