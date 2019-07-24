use std::io;

fn print_description(_app_id: String) -> io::Result<()> {
    println!("0/1: <luxtorpeda game package>");
    Ok(())
}

pub fn iscriptevaluator(args: &[&str]) -> io::Result<()> {
    match args {
        ["--get-current-step", steam_app_id] => {
            let app_id = steam_app_id.to_string();
            return print_description(app_id);
        }
        ["--get-current-step"] => return Ok(()),
        _ => {}
    }
    println!("fake script evaluator");
    Ok(())
}
