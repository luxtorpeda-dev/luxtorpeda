use std::env;
use std::io;
use std::io::{Error, ErrorKind};
use std::path;
use std::fs;
use std::process::Command;

mod pid_file;
mod user_env;

extern crate json;


fn usage() {
    println!("usage: lux [run | wait-before-run]");
}

fn run(arg_0: &String, args: &[String]) -> io::Result<()> {
    let _pid_file = pid_file::new()?;
    let app_id = user_env::steam_app_id();
    let tool_path = path::Path::new(arg_0);

    println!("working dir: {:?}", env::current_dir());
    println!("tool dir: {:?}", tool_path.parent());
    println!("args: {:?}", args);
    println!("steam_app_id: {:?}", &app_id);

    let packages_json_file = tool_path.parent().unwrap().join("packages.json");
    let json_str = fs::read_to_string(packages_json_file).unwrap();
    let parsed = json::parse(json_str.as_ref()).unwrap();
    let game_info = &parsed[app_id];

    if game_info.is_null() {
        return Err(Error::new(ErrorKind::Other, "Unknown app_id"));
    }

    println!("json:");
    println!("{:#}", game_info);

    if !game_info["zipfile"].is_null() {
        let zip = game_info["zipfile"].to_string();
        let dist_zip = tool_path.parent().unwrap().join(zip);
        Command::new("unzip")
            .arg("-uo")
            .arg(dist_zip)
            .status()
            .expect("failed to execute process");
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

    match cmd.as_str() {
        "run" => run(&args[0], cmd_args),
        "wait-before-run" => {
            pid_file::wait_while_exists();
            run(&args[0], cmd_args)
        }
        _ => {
            usage();
            std::process::exit(1)
        }
    }
}
