extern crate xdg;
extern crate users;

use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use users::get_current_uid;


/// Assure, that XDG_RUNTIME_DIR is set with correct access mode.
///
pub fn assure_xdg_runtime_dir() -> Result<(), std::io::Error> {
    let xdg_dirs = xdg::BaseDirectories::new()?;

    if xdg_dirs.has_runtime_directory() {
        Ok(())
    } else {
        let runtime_dir = format!("/tmp/luxtorpeda_{}", get_current_uid());
        if !Path::new(&runtime_dir).is_dir() {
            fs::create_dir(&runtime_dir)?;
        }
        fs::set_permissions(&runtime_dir, fs::Permissions::from_mode(0o700))?;
        env::set_var("XDG_RUNTIME_DIR", &runtime_dir);
        Ok(())
    }
}

/// Return `SteamAppId` environment variable or `"0"` as a fallback.
///
/// Steam uses this variable to identify games that originate from the
/// Steam store.  Non-Steam games have this variable set to `"0"` and
/// use `SteamGameId` environment variable instead.
///
pub fn steam_app_id() -> String {
    match env::var("SteamAppId") {
        Ok(app_id) => app_id,
        Err(_) => "0".to_string(),
    }
}
