extern crate xdg;

use log::warn;
use std::env;
use std::path::Path;
use std::path::PathBuf;

use crate::command;

static LUX_TOOL_DIR: &str = "LUX_TOOL_DIR";
static STEAM_APPID: &str = "SteamAppId";
static LUX_CONTROLLER: &str = "LUX_CONTROLLER";
static STEAM_COMPAT_CLIENT_INSTALL_PATH: &str = "STEAM_COMPAT_CLIENT_INSTALL_PATH";

pub fn assure_tool_dir(arg0: &str) -> Result<(), std::io::Error> {
    let tool_path = Path::new(arg0);
    env::set_var(LUX_TOOL_DIR, tool_path.parent().unwrap());
    Ok(())
}

/// Return `SteamAppId` environment variable or `"0"` as a fallback.
///
/// Steam uses this variable to identify games that originate from the
/// Steam store.  Non-Steam games have this variable set to `"0"` and
/// use `SteamGameId` environment variable instead.
///
pub fn steam_app_id() -> String {
    let manual_download_check = manual_download_app_id();
    if !manual_download_check.is_empty() {
        return manual_download_check;
    }

    match env::var(STEAM_APPID) {
        Ok(app_id) => app_id,
        Err(_) => "0".to_string(),
    }
}

pub fn steam_install_path() -> Option<String> {
    match env::var(STEAM_COMPAT_CLIENT_INSTALL_PATH) {
        Ok(path) => Some(path),
        Err(err) => {
            warn!("steam_install_path err: {}", err);
            None
        }
    }
}

pub fn manual_download_app_id() -> String {
    let env_args: Vec<String> = env::args().collect();
    let args: Vec<&str> = env_args.iter().map(|a| a.as_str()).collect();
    if !args.is_empty() && args.len() > 1 {
        let cmd = args[1];
        let cmd_args = &args[2..];

        if cmd == "manual-download" {
            if !cmd_args.is_empty() {
                let app_id = cmd_args[0];
                return app_id.to_string();
            } else {
                command::usage();
                std::process::exit(0)
            }
        }
    }

    String::new()
}

pub fn tool_dir() -> PathBuf {
    match env::var(LUX_TOOL_DIR) {
        Ok(path) => PathBuf::from(&path),
        Err(_) => env::current_dir().unwrap(),
    }
}

pub fn set_env_var(key: &str, value: &str) {
    env::set_var(key, value);
}

pub fn set_controller_var(value: &str) {
    set_env_var(LUX_CONTROLLER, value);
}
