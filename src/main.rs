use std::env;

mod user_env;

fn usage() {
    println!("usage: lux [run | wait-before-run]");
}

fn wait() {
}

fn run(args: &[String]) {
    println!("working dir: {:?}", env::current_dir());
    println!("args: {:?}", args);
    println!("steam_app_id: {:?}", user_env::steam_app_id())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        usage();
        return;
    }

    let cmd = &args[1];
    let cmd_args = &args[2..];

    match cmd.as_str() {
        "run" => run(cmd_args),
        "wait-before-run" => { wait(); run(cmd_args) },
        _ => { usage(); ::std::process::exit(1) },
    }
}
