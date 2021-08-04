use std::io;
use std::env;
use std::io::{Error, ErrorKind};
use std::process::Command;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::fs;

use crate::user_env;

fn get_zenity_path() -> Result<String, Error>  {
    let zenity_path = match env::var("STEAM_ZENITY") {
        Ok(s) => s,
        Err(_) => {
            return Err(Error::new(ErrorKind::Other, "Path could not be found"));
        }
    };

    return Ok(zenity_path);
}

fn get_current_desktop() -> Option<String> {
    let current_desktop = match env::var("XDG_CURRENT_DESKTOP") {
        Ok(s) => s,
        Err(_) => {
            return None
        }
    };

    return Some(current_desktop);
}

fn active_dialog_command() -> io::Result<String> {
    let config_json_file = user_env::tool_dir().join("config.json");
    let config_json_str = fs::read_to_string(config_json_file)?;
    let config_parsed = json::parse(&config_json_str).unwrap();

    if config_parsed["active_dialog_command"].is_null() {
        println!("active_dialog_command. config not found, assuming zenity");
        Ok("zenity".to_string())
    } else {
        let active_dialog_command = &config_parsed["active_dialog_command"];
        let active_dialog_command_str = active_dialog_command.to_string();
        println!("active_dialog_command. active_dialog_command_str: {:?}", active_dialog_command_str);

        if active_dialog_command_str != "default" {
            Ok(active_dialog_command_str)
        }
        else {
            match get_current_desktop() {
                Some(current_desktop) => {
                    println!("active_dialog_command. current_desktop: {:?}", current_desktop);
                    if current_desktop == "KDE" {
                        println!("active_dialog_command. current desktop of kde found, assuming kdialog");
                        Ok("kdialog".to_string())
                    } else {
                        println!("active_dialog_command. current desktop unknown, assuming zenity");
                        Ok("zenity".to_string())
                    }
                },
                None => {
                    println!("active_dialog_command. no current desktop found, assuming zenity");
                    Ok("zenity".to_string())
                }
            }
        }
    }
}

pub fn show_error(title: &String, error_message: &String) -> io::Result<()> {
    if active_dialog_command()? == "kdialog" {
        let command: Vec<String> = vec![
            "--error".to_string(),
            error_message.to_string(),
            "--title".to_string(),
            title.to_string()
        ];

        Command::new("kdialog")
            .args(&command)
            .status()
            .expect("failed to show kdialog error");
    } else {
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
    }

    Ok(())
}

pub fn show_choices(title: &str, column: &str, choices: &Vec<String>) -> io::Result<String> {
    if active_dialog_command()? == "kdialog" {
        let mut list_command: Vec<String> = vec![
            "--geometry".to_string(),
            "350x300".to_string(),
            "--title".to_string(),
            column.to_string(),
            "--radiolist".to_string(),
            title.to_string()
        ];

        for entry in choices {
            list_command.push(entry.to_string());
            list_command.push(entry.to_string());
            list_command.push("off".to_string());
        }

        let choice = Command::new("kdialog")
            .args(&list_command)
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
    } else {
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
}

pub fn show_file_with_confirm(title: &str, file_path: &str) -> io::Result<()> {
    let mut file = File::open(&file_path)?;
    let mut file_buf = vec![];
    file.read_to_end(&mut file_buf)?;
    let file_str = String::from_utf8_lossy(&file_buf);

    let mut converted_file = File::create("converted.txt")?;
    converted_file.write_all(file_str.as_bytes())?;

    if active_dialog_command()? == "kdialog" {
        let command: Vec<String> = vec![
            "--geometry".to_string(),
            "400x600".to_string(),
            "--textbox".to_string(),
            "converted.txt".to_string(),
            "--title".to_string(),
            title.to_string()
        ];

        Command::new("kdialog")
            .args(&command)
            .status()
            .expect("failed to show kdialog error");

        match show_question(&std::format!("{} Confirmation", title).to_string(), "I have read and accepted the terms.") {
            Some(_) => {
                Ok(())
            },
            None => {
                return Err(Error::new(ErrorKind::Other, "file confirmation denied"))
            }
        }
    } else {
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
        }
        else {
            Ok(())
        }
    }
}

pub fn show_question(title: &str, text: &str) -> Option<()> {
    if active_dialog_command().ok()? == "kdialog" {
        let command: Vec<String> = vec![
            "--yesno".to_string(),
            text.to_string(),
            "--title".to_string(),
            title.to_string()
        ];

        let question = Command::new("kdialog")
            .args(&command)
            .status()
            .expect("failed to show kdialog error");

        if question.success() {
            Some(())
        } else {
            return None
        }
    } else {
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
}
