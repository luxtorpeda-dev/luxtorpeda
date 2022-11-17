extern crate users;
extern crate xdg;

use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;

use crate::command;
use users::get_current_uid;

static XDG_RUNTIME_DIR: &str = "XDG_RUNTIME_DIR";
static LUX_TOOL_DIR: &str = "LUX_TOOL_DIR";
static STEAM_APPID: &str = "SteamAppId";
static LUX_CONTROLLER: &str = "LUX_CONTROLLER";

/// Assure, that XDG_RUNTIME_DIR is set with correct access mode.
///
pub fn assure_xdg_runtime_dir() -> Result<(), std::io::Error> {
    let xdg_dirs = xdg::BaseDirectories::new()?;
    if xdg_dirs.has_runtime_directory() {
        return Ok(());
    }
    let runtime_dir = format!("/tmp/luxtorpeda_{}", get_current_uid());
    if !Path::new(&runtime_dir).is_dir() {
        fs::create_dir(&runtime_dir)?;
    }
    fs::set_permissions(&runtime_dir, fs::Permissions::from_mode(0o700))?;
    env::set_var(XDG_RUNTIME_DIR, &runtime_dir);
    Ok(())
}

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
