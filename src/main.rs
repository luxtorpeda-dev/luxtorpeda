use std::env;
use std::io;
use std::io::{Error, ErrorKind};
use std::path;
use std::process::Command;

mod pid_file;
mod user_env;

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

    // TODO download packages:
    // https://luxtorpeda.gitlab.io/packages/openjk/
    // https://luxtorpeda.gitlab.io/packages/ioq3/
    // https://luxtorpeda.gitlab.io/packages/openxcom/

    match app_id.as_ref() {
        // Quake III Arena
        "2200" => {
            let dist_zip = tool_path.parent().unwrap().join("2200.zip");
            Command::new("unzip")
                .arg("-uo")
                .arg(dist_zip)
                .status()
                .expect("failed to execute process");
            Command::new("./ioquake3.x86_64")
                .arg("+r_mode")
                .arg("-2")
                .status()
                .expect("failed to execute process");
            Ok(())
        }

        // STAR WARS™ Jedi Knight: Jedi Academy™
        "6020" => {
            let dist_zip = tool_path.parent().unwrap().join("6020.zip");
            Command::new("unzip")
                .arg("-uo")
                .arg(dist_zip)
                .status()
                .expect("failed to execute process");
            // Steam changes working directory to "GameData"
            // TODO handle MP
            Command::new("./openjk_sp.x86_64")
                .status()
                .expect("failed to execute process");
            Ok(())
        }

        // Doki Doki Literature Club
        "698780" => {
            Command::new("./DDLC.sh")
                .status()
                .expect("failed to execute process");
            Ok(())
        }

        _ => Err(Error::new(ErrorKind::Other, "I don't know this app_id!")),
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
