use std::io;

fn print_current_step() -> io::Result<()> {
    println!("0/1: <luxtorpeda package>");
    Ok(())
}

pub fn iscriptevaluator(args: &[&str]) -> io::Result<()> {
    match args {
        ["--get-current-step", _] => return print_current_step(),
        _ => {}
    }
    // rest of logic here
    Ok(())
}
