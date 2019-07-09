use std::env;

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
