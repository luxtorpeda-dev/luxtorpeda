use std::env;

fn usage() {
    println!("usage: lux [run | wait-before-run]");
}

fn wait() {
}

fn run(args: &[String]) {
    println!("{:?}", args);
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
        _ => usage(),
    };
}
