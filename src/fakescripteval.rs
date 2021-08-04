extern crate regex;
extern crate steamy_vdf as vdf;

use regex::Regex;
use std::io;
use std::io::{Error, ErrorKind};
use std::path::Path;
use std::path::PathBuf;

use crate::ipc;

fn get_vdf_path(exe: &str, script_vdf: &str)  -> io::Result<PathBuf> {
    let exe_path = Path::new(exe);
    let exe_dir = exe_path.parent().expect("Executable must be in some directory");
    let exe_dir_up = exe_dir.parent().expect("Executable must be in some directory");

    if !exe_dir_up.exists() {
        let exe_dir_up_str = str::replace(exe_dir_up.to_str().unwrap(), "steam", "Steam");
        let final_exe_dir = Path::new(&exe_dir_up_str);
        let script_vdf_fixed = str::replace(script_vdf, "\\", "/");
        let vdf_path = final_exe_dir.join(script_vdf_fixed);
        return Ok(vdf_path);
    } else {
        let exe_dir_up_str = exe_dir_up.to_str().unwrap();
        let final_exe_dir = Path::new(exe_dir_up_str);
        let script_vdf_fixed = str::replace(script_vdf, "\\", "/");
        let vdf_path = final_exe_dir.join(script_vdf_fixed);
        return Ok(vdf_path);
    }
}

fn check_for_uninstall(exe: &str, script_vdf: &str) -> io::Result<()> {
    let vdf_path = get_vdf_path(exe, script_vdf)?;
    println!("check_for_uninstall vdf_path: {:?}", vdf_path);

    if vdf_path.exists() {
        let config = vdf::load(vdf_path).unwrap();
        let uninstall_flag_ref = config.lookup("evaluatorscript.0.uninstall");
        if !uninstall_flag_ref.is_none() {
            let uninstall_flag = uninstall_flag_ref.unwrap().as_str().unwrap();
            println!("check_for_uninstall uninstall_flag: {:?}", uninstall_flag);

            if uninstall_flag == "1" {
                return Err(Error::new(ErrorKind::Other, "User uninstalling, so should exit"))
            }
        } else {
            println!("check_for_uninstall uninstall_flag not found");
        }
    } else {
        println!("check_for_uninstall vdf not found");
    }

    return Ok(())
}

fn extract_steam_app_id(input: &str) -> Option<&str> {
    lazy_static! {
        static ref RE: Regex = Regex::new(r".*script_(?P<id>\d+)\.vdf").unwrap();
    }
    RE.captures(input)
        .and_then(|cap| cap.name("id").map(|x| x.as_str()))
}

pub fn iscriptevaluator(exe: &str, args: &[&str]) -> io::Result<()> {
    match args {
        ["--get-current-step", steam_app_id] => {
            let app_id = steam_app_id.to_string();
            ipc::query_status(app_id);
            Ok(())
        }
        [script_vdf] => {
            let steam_app_id = extract_steam_app_id(script_vdf);
            match steam_app_id {
                Some(app_id) => {
                    match check_for_uninstall(exe, script_vdf) {
                        Ok(()) => {
                            println!("iscriptevaluator ignoring for {}", app_id);
                            Ok(())
                        },
                        Err(err) => {
                            let error_message = std::format!("script_vdf: {:?}", err);
                            println!("{:?}", error_message);
                            return Err(err);
                        }
                    }
                },
                None => Err(Error::new(ErrorKind::Other, "Unknown app_id")),
            }
        }
        _ => Ok(()),
    }
}
