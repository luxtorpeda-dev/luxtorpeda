use std::io;

fn print_current_step() -> io::Result<()> {
    println!("0/1: <luxtorpeda game package>");
    Ok(())
}

pub fn iscriptevaluator(args: &[&str]) -> io::Result<()> {
    match args {
        ["--get-current-step", _script] => return print_current_step(),
        ["--get-current-step"] => return print_current_step(),
        _ => {}
    }
    println!("fake script evaluator");
    Ok(())
}
