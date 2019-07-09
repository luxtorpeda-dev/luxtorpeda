use std::env;

mod pid_file;
mod user_env;

fn usage() {
    println!("usage: lux [run | wait-before-run]");
}

fn wait() {}

fn run(args: &[String]) {
    let _pid_file = pid_file::new();
    println!("working dir: {:?}", env::current_dir());
    println!("args: {:?}", args);
    println!("steam_app_id: {:?}", user_env::steam_app_id());
}

fn main() -> Result<(), std::io::Error> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        usage();
        std::process::exit(0)
    }

    let cmd = &args[1];
    let cmd_args = &args[2..];

    user_env::assure_xdg_runtime_dir()?;

    match cmd.as_str() {
        "run" => {
            run(cmd_args);
            Ok(())
        }
        "wait-before-run" => {
            wait();
            run(cmd_args);
            Ok(())
        }
        _ => {
            usage();
            std::process::exit(1)
        }
    }
}
