use std::io;
use std::env;
use std::io::{Error, ErrorKind};
use std::process::Command;
use std::process::Child;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::fs;
use std::process::Stdio;

use crate::user_env;

pub enum ProgressCreateOutput {
    KDialog(String),
    Zenity(Child),
}

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

fn active_dialog_command(silent: bool) -> io::Result<String> {
    let config_json_file = user_env::tool_dir().join("config.json");
    let config_json_str = fs::read_to_string(config_json_file)?;
    let config_parsed = json::parse(&config_json_str).unwrap();

    if config_parsed["active_dialog_command"].is_null() {
        if !silent {
            println!("active_dialog_command. config not found, assuming zenity");
        }
        Ok("zenity".to_string())
    } else {
        let active_dialog_command = &config_parsed["active_dialog_command"];
        let active_dialog_command_str = active_dialog_command.to_string();
        if !silent {
            println!("active_dialog_command. active_dialog_command_str: {:?}", active_dialog_command_str);
        }

        if active_dialog_command_str != "default" {
            Ok(active_dialog_command_str)
        }
        else {
            match get_current_desktop() {
                Some(current_desktop) => {
                    if !silent {
                        println!("active_dialog_command. current_desktop: {:?}", current_desktop);
                    }
                    if current_desktop == "KDE" {
                        if !silent {
                            println!("active_dialog_command. current desktop of kde found, assuming kdialog");
                        }
                        Ok("kdialog".to_string())
                    } else {
                        if !silent {
                            println!("active_dialog_command. current desktop unknown, assuming zenity");
                        }
                        Ok("zenity".to_string())
                    }
                },
                None => {
                    if !silent {
                        println!("active_dialog_command. no current desktop found, assuming zenity");
                    }
                    Ok("zenity".to_string())
                }
            }
        }
    }
}

pub fn show_error(title: &String, error_message: &String) -> io::Result<()> {
    if active_dialog_command(false)? == "kdialog" {
        let command: Vec<String> = vec![
            "--error".to_string(),
            error_message.to_string(),
            "--title".to_string(),
            title.to_string()
        ];

        Command::new("kdialog")
            .args(&command)
            .env("LD_PRELOAD", "")
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
            .env("LD_PRELOAD", "")
            .status()
            .expect("failed to show zenity error");
    }

    Ok(())
}

pub fn show_choices(title: &str, column: &str, choices: &Vec<String>) -> io::Result<String> {
    if active_dialog_command(false)? == "kdialog" {
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
            .env("LD_PRELOAD", "")
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
            .env("LD_PRELOAD", "")
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

    if active_dialog_command(false)? == "kdialog" {
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
            .env("LD_PRELOAD", "")
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
            .env("LD_PRELOAD", "")
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
    if active_dialog_command(false).ok()? == "kdialog" {
        let command: Vec<String> = vec![
            "--yesno".to_string(),
            text.to_string(),
            "--title".to_string(),
            title.to_string()
        ];

        let question = Command::new("kdialog")
            .args(&command)
            .env("LD_PRELOAD", "")
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
            .env("LD_PRELOAD", "")
            .status()
            .expect("failed to show question");

        if question.success() {
            Some(())
        } else {
            return None
        }
    }
}

pub fn start_progress(title: &str, status: &str, interval: usize) -> io::Result<ProgressCreateOutput> {
    if active_dialog_command(false)? == "kdialog" {
        let progress_command: Vec<String> = vec![
            "--geometry".to_string(),
            "600x200".to_string(),
            "--title".to_string(),
            title.to_string(),
            "--progressbar".to_string(),
            status.to_string(),
            (interval).to_string()
        ];

        println!("progress_command  {:?}", progress_command);

        let progress = Command::new("kdialog")
            .args(&progress_command)
            .env("LD_PRELOAD", "")
            .output()
            .expect("failed to show progress");

        if !progress.status.success() {
            return Err(Error::new(ErrorKind::Other, "progress start failed"));
        }

        let progress_id = match String::from_utf8(progress.stdout) {
            Ok(s) => String::from(s.trim()),
            Err(_) => {
                return Err(Error::new(ErrorKind::Other, "Failed to parse progress_id"));
            }
        };

        let progress_id_array = progress_id.split(" ").collect::<Vec<_>>();
        let progress_disable_cancel_command: Vec<String> = vec![
            progress_id_array[0].to_string(),
            progress_id_array[1].to_string(),
            "showCancelButton".to_string(),
            "false".to_string()
        ];

        println!("progress_disable_cancel_command {:?}", progress_disable_cancel_command);

        let progress_disable_cancel = Command::new("qdbus")
            .env("LD_PRELOAD", "")
            .args(&progress_disable_cancel_command)
            .output()
            .expect("failed to update disable cancel progress");

        if !progress_disable_cancel.status.success() {
            return Err(Error::new(ErrorKind::Other, "progress update disable cancel failed"));
        }

        Ok(ProgressCreateOutput::KDialog(progress_id))
    } else {
         let progress_command: Vec<String> = vec![
            "--progress".to_string(),
            "--no-cancel".to_string(),
            std::format!("--title={}", title).to_string(),
            std::format!("--percentage=0").to_string(),
            std::format!("--text={}", status).to_string()
        ];

        println!("progress_command {:?}", progress_command);

        let zenity_path = match get_zenity_path() {
            Ok(s) => s,
            Err(_) => {
                return Err(Error::new(ErrorKind::Other, "zenity path not found"))
            }
        };

        let progress = Command::new(zenity_path)
            .args(&progress_command)
            .env("LD_PRELOAD", "")
            .stdin(Stdio::piped())
            .spawn()
            .unwrap();

        Ok(ProgressCreateOutput::Zenity(progress))
    }
}

pub fn progress_text_change(title: &str, progress_ref: &mut ProgressCreateOutput) -> io::Result<()> {
    if let ProgressCreateOutput::KDialog(progress_id) = progress_ref {
        let progress_id_array = progress_id.split(" ").collect::<Vec<_>>();
        if progress_id_array.len() == 2 {
            let progress_command_label: Vec<String> = vec![
                progress_id_array[0].to_string(),
                progress_id_array[1].to_string(),
                "setLabelText".to_string(),
                title.to_string()
            ];

            println!("progress_command_label {:?}", progress_command_label);

            let progress_label = Command::new("qdbus")
                .args(&progress_command_label)
                .env("LD_PRELOAD", "")
                .output()
                .expect("failed to update progress label");

            if !progress_label.status.success() {
                return Err(Error::new(ErrorKind::Other, "progress update label failed"));
            }
        }

        Ok(())
    } else if let ProgressCreateOutput::Zenity(ref mut progress) = progress_ref {
        {
            let stdin = progress.stdin.as_mut().expect("Failed to open stdin");
            stdin.write_all(std::format!("# {}\n", title).as_bytes()).expect("Failed to write to stdin");
            drop(stdin);
        }
        Ok(())
    } else {
        return Err(Error::new(ErrorKind::Other, "Progress not implemented"));
    }
}

pub fn progress_change(value: i64, progress_ref: &mut ProgressCreateOutput) -> io::Result<()> {
    if let ProgressCreateOutput::KDialog(progress_id) = progress_ref {
        let progress_id_array = progress_id.split(" ").collect::<Vec<_>>();
        if progress_id_array.len() == 2 {
            let progress_command: Vec<String> = vec![
                progress_id_array[0].to_string(),
                progress_id_array[1].to_string(),
                "Set".to_string(),
                "".to_string(),
                "value".to_string(),
                value.to_string()
            ];

            let progress = Command::new("qdbus")
                .args(&progress_command)
                .env("LD_PRELOAD", "")
                .output()
                .expect("failed to update progress");

            if !progress.status.success() {
                return Err(Error::new(ErrorKind::Other, "progress update failed"));
            }
        }
        Ok(())
    } else if let ProgressCreateOutput::Zenity(ref mut progress) = progress_ref {
        {
            let mut final_value = value;
            if final_value == 100 {
                final_value = 99;
            }
            let stdin = progress.stdin.as_mut().expect("Failed to open stdin");
            stdin.write_all(std::format!("{}\n", final_value).as_bytes()).expect("Failed to write to stdin");
            drop(stdin);
        }
        Ok(())
    } else {
        return Err(Error::new(ErrorKind::Other, "Progress not implemented"));
    }
}

pub fn progress_close(progress_ref: &mut ProgressCreateOutput) -> io::Result<()> {
    if let ProgressCreateOutput::KDialog(progress_id) = progress_ref {
        let progress_id_array = progress_id.split(" ").collect::<Vec<_>>();
        if progress_id_array.len() == 2 {
            let progress_command: Vec<String> = vec![
                progress_id_array[0].to_string(),
                progress_id_array[1].to_string(),
                "close".to_string()
            ];

            let progress = Command::new("qdbus")
                .args(&progress_command)
                .env("LD_PRELOAD", "")
                .output()
                .expect("failed to close progress");

            if !progress.status.success() {
                return Err(Error::new(ErrorKind::Other, "progress close failed"));
            }
        }

        Ok(())
    } else if let ProgressCreateOutput::Zenity(ref mut progress) = progress_ref {
        progress.kill().expect("command wasn't running");
        Ok(())
    } else {
        return Err(Error::new(ErrorKind::Other, "Progress not implemented"));
    }
}
