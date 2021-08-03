use std::io;
use std::env;
use std::io::{Error, ErrorKind};
use std::process::Command;
use std::fs::File;
use std::io::Read;
use std::io::Write;

fn get_zenity_path() -> Result<String, Error>  {
    let zenity_path = match env::var("STEAM_ZENITY") {
        Ok(s) => s,
        Err(_) => {
            return Err(Error::new(ErrorKind::Other, "Path could not be found"));
        }
    };

    return Ok(zenity_path);
}

pub fn show_error(title: &String, error_message: &String) -> io::Result<()> {
    let zenity_path = match get_zenity_path() {
        Ok(s) => s,
        Err(_) => {
            return Err(Error::new(ErrorKind::Other, "zenity path not found"))
        }
    };

    let zenity_command: Vec<String> = vec![
        "--error".to_string(),
        std::format!("--text={}", error_message).to_string(),
        std::format!("--title={}", title).to_string()
    ];

    Command::new(zenity_path)
        .args(&zenity_command)
        .status()
        .expect("failed to show zenity error");

    Ok(())
}

pub fn show_choices(title: &str, column: &str, choices: &Vec<String>) -> io::Result<String> {
    let mut zenity_list_command: Vec<String> = vec![
        "--list".to_string(),
        std::format!("--title={0}", title),
        std::format!("--column={0}", column),
        "--hide-header".to_string()
    ];

    for entry in choices {
        zenity_list_command.push(entry.to_string());
    }

    let zenity_path = match get_zenity_path() {
        Ok(s) => s,
        Err(_) => {
            return Err(Error::new(ErrorKind::Other, "zenity path not found"))
        }
    };

    let choice = Command::new(zenity_path)
        .args(&zenity_list_command)
        .output()
        .expect("failed to show choices");

    if !choice.status.success() {
        return Err(Error::new(ErrorKind::Other, "dialog was rejected"));
    }

    let choice_name = match String::from_utf8(choice.stdout) {
        Ok(s) => String::from(s.trim()),
        Err(_) => {
            return Err(Error::new(ErrorKind::Other, "Failed to parse choice name"));
        }
    };

    Ok(choice_name)
}

pub fn show_file_with_confirm(title: &str, file_path: &str) -> io::Result<()> {
    let mut file = File::open(&file_path)?;
    let mut file_buf = vec![];
    file.read_to_end(&mut file_buf)?;
    let file_str = String::from_utf8_lossy(&file_buf);

    let mut converted_file = File::create("converted.txt")?;
    converted_file.write_all(file_str.as_bytes())?;

    let zenity_path = match get_zenity_path() {
        Ok(s) => s,
        Err(_) => {
            return Err(Error::new(ErrorKind::Other, "zenity path not found"))
        }
    };

    let choice = Command::new(zenity_path)
        .args(&[
            "--text-info",
            &std::format!("--title={0}", title).to_string(),
            "--filename=converted.txt"])
        .status()
        .expect("failed to show file with confirm");

    if !choice.success() {
       return Err(Error::new(ErrorKind::Other, "dialog was rejected"));
    } else {
        Ok(())
    }
}

pub fn show_question(title: &str, text: &str) -> Option<()> {
    let zenity_command: Vec<String> = vec![
        "--question".to_string(),
        std::format!("--text={}", &text),
        std::format!("--title={}", &title)
    ];

    let zenity_path = match get_zenity_path() {
        Ok(s) => s,
        Err(_) => {
            return None
        }
    };

    let question = Command::new(zenity_path)
        .args(&zenity_command)
        .status()
        .expect("failed to show question");

    if question.success() {
        Some(())
    } else {
       return None
    }
}
