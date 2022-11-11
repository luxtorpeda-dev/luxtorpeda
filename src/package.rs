#![allow(clippy::or_fun_call)]

extern crate reqwest;
extern crate tar;
extern crate xz2;

use bzip2::read::BzDecoder;
use flate2::read::GzDecoder;
use futures_util::StreamExt;
use log::{error, info};
use regex::Regex;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha1::{Digest, Sha1};
use std::cmp::min;
use std::collections::HashMap;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Write;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use tar::Archive;
use tokio::runtime::Runtime;
use xz2::read::XzDecoder;

use crate::user_env;

extern crate steamlocate;
use steamlocate::SteamDir;

static LUX_DISABLE_DEFAULT_CONFIRM: &str = "LUX_DISABLE_DEFAULT_CONFIRM";

pub fn place_cached_file(app_id: &str, file: &str) -> io::Result<PathBuf> {
    let xdg_dirs = xdg::BaseDirectories::new().unwrap();
    let path_str = format!("luxtorpeda/{}/{}", app_id, file);
    xdg_dirs.place_cache_file(path_str)
}

fn find_cached_file(app_id: &str, file: &str) -> Option<PathBuf> {
    let xdg_dirs = xdg::BaseDirectories::new().unwrap();
    let path_str = format!("luxtorpeda/{}/{}", app_id, file);
    xdg_dirs.find_cache_file(path_str)
}

// Try to create dirs of path recursively,
// if that fails, try to show a helpful UI message
fn create_dir_or_show_error(path: &impl AsRef<Path>) {
    let err = match fs::create_dir_all(path) {
        Ok(()) => return,
        Err(err) => err,
    };

    let path = path.as_ref();
    let mut msg = format!(
        "Error creating directory {:?} (or one of its parents): {:?}",
        path, err
    );
    if err.kind() == ErrorKind::AlreadyExists && !path.exists() {
        msg += r"
Cross filesystem interaction detected.
It seems this folder is on a filesystem to which the Steam runtime prevents access.
Try changing Launch Options in Steam to:
STEAM_COMPAT_MOUNTS=/path/to/other/filesystem %command%";
        // Steam runtime restrictions + symlinks are weird.
        // Because a symlink acts as its target in most situations, from inside the
        // runtime environment, symlinks to forbidden filesystems look as if they simply
        // do not exist. That's ok for read ops. But we want to create a hierarchy if it
        // does not exist. So create_dir_all happily tries to create a directory where
        // the symlink is. And that's when the OS says "nope, there's already something
        // here".
    }
    //let _ = show_error("Setup Error", msg.as_str());
    //panic!("{}", msg);
}

pub fn place_config_file(app_id: &str, file: &str) -> io::Result<PathBuf> {
    let xdg_dirs = xdg::BaseDirectories::new().unwrap();
    let path_str = format!("luxtorpeda/{}/{}", app_id, file);
    xdg_dirs.place_config_file(path_str)
}

pub fn path_to_packages_file() -> PathBuf {
    let xdg_dirs = xdg::BaseDirectories::new().unwrap();
    let config_home = xdg_dirs.get_cache_home();
    let folder_path = config_home.join("luxtorpeda");
    create_dir_or_show_error(&folder_path);
    folder_path.join("packagesruntime.json")
}

pub fn path_to_cache() -> PathBuf {
    let xdg_dirs = xdg::BaseDirectories::new().unwrap();
    let cache_home = xdg_dirs.get_cache_home();
    let folder_path = cache_home.join("luxtorpeda");
    create_dir_or_show_error(&folder_path);
    folder_path
}

pub fn path_to_config() -> PathBuf {
    let xdg_dirs = xdg::BaseDirectories::new().unwrap();
    let config_home = xdg_dirs.get_config_home();
    let folder_path = config_home.join("luxtorpeda");
    create_dir_or_show_error(&folder_path);
    folder_path
}

pub fn find_user_packages_file() -> Option<PathBuf> {
    let xdg_dirs = xdg::BaseDirectories::new().unwrap();
    let path_str = "luxtorpeda/user-packages.json";
    xdg_dirs.find_config_file(path_str)
}

pub fn place_state_file(file: &str) -> io::Result<PathBuf> {
    let xdg_dirs = xdg::BaseDirectories::new().unwrap();
    let path_str = format!("luxtorpeda/{}", file);
    xdg_dirs.place_state_file(path_str)
}

pub fn convert_notice_to_str(notice_item: &json::JsonValue, notice_map: &json::JsonValue) -> String {
    let mut notice = String::new();

    if !notice_item.is_null() {
        if !notice_item["label"].is_null() {
            notice = notice_item["label"].to_string();
        } else if !notice_item["value"].is_null() {
            if !notice_map[notice_item["value"].to_string()].is_null() {
                notice = notice_map[notice_item["value"].to_string()].to_string();
            } else {
                notice = notice_item["value"].to_string();
            }
        } else if !notice_item["key"].is_null() {
            if !notice_map[notice_item["key"].to_string()].is_null() {
                notice = notice_map[notice_item["key"].to_string()].to_string();
            } else {
                notice = notice_item["key"].to_string();
            }
        }
    }

    notice
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CmdReplacement {
    #[serde(with = "serde_regex")]
    pub match_cmd: Regex,
    pub cmd: String,
    pub args: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PackageMetadata {
    pub engine_version: String,
    pub commands: Vec<CmdReplacement>,
}

pub struct PackageInfo {
    pub name: String,
    pub url: String,
    pub file: String,
    pub cache_by_name: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChoiceInfo {
    pub name: String,
    pub notices: Vec<String>,
}

pub fn generate_hash_from_file_path(file_path: &Path) -> io::Result<String> {
    let json_str = fs::read_to_string(file_path)?;
    let mut hasher = Sha1::new();
    hasher.update(json_str);
    let hash_result = hasher.finalize();
    let hash_str = hex::encode(hash_result);
    Ok(hash_str)
}

pub fn get_remote_packages_hash(remote_hash_url: &str) -> Option<String> {
    let remote_hash_response = match reqwest::blocking::get(remote_hash_url) {
        Ok(s) => s,
        Err(err) => {
            error!("get_remote_packages_hash error in get: {:?}", err);
            return None;
        }
    };

    let remote_hash_str = match remote_hash_response.text() {
        Ok(s) => s,
        Err(err) => {
            error!("get_remote_packages_hash error in text: {:?}", err);
            return None;
        }
    };

    Some(remote_hash_str)
}

pub fn json_to_downloads(app_id: &str, game_info: &json::JsonValue) -> io::Result<Vec<PackageInfo>> {
    let mut downloads: Vec<PackageInfo> = Vec::new();

    for entry in game_info["download"].members() {
        if entry["name"].is_null() || entry["url"].is_null() || entry["file"].is_null() {
            return Err(Error::new(ErrorKind::Other, "missing download info"));
        }

        let name = entry["name"].to_string();
        let url = entry["url"].to_string();
        let file = entry["file"].to_string();
        let mut cache_by_name = false;

        let mut cache_dir = app_id;
        if entry["cache_by_name"] == true {
            cache_dir = &name;
            cache_by_name = true;
        }

        if find_cached_file(cache_dir, file.as_str()).is_some() {
            info!("{} found in cache (skip)", file);
            continue;
        }

        downloads.push(PackageInfo {
            name,
            url,
            file,
            cache_by_name,
        });
    }
    Ok(downloads)
}

pub fn update_packages_json() -> io::Result<()> {
        let config_json_file = user_env::tool_dir().join("config.json");
        let config_json_str = fs::read_to_string(config_json_file)?;
        let config_parsed = json::parse(&config_json_str).unwrap();

        let should_do_update = &config_parsed["should_do_update"];
        if should_do_update != true {
            return Ok(());
        }

        let packages_json_file = path_to_packages_file();
        let mut should_download = true;
        let mut remote_hash_str: String = String::new();

        let remote_path = "packagesruntime";

        let remote_hash_url = std::format!("{0}/{1}.hash", &config_parsed["host_url"], remote_path);
        match get_remote_packages_hash(&remote_hash_url) {
            Some(tmp_hash_str) => {
                remote_hash_str = tmp_hash_str;
            }
            None => {
                info!("update_packages_json in get_remote_packages_hash call. received none");
                should_download = false;
            }
        }

        if should_download {
            if !Path::new(&packages_json_file).exists() {
                should_download = true;
                info!(
                    "update_packages_json. {:?} does not exist",
                    packages_json_file
                );
            } else {
                let hash_str = generate_hash_from_file_path(&packages_json_file)?;
                info!("update_packages_json. found hash: {}", hash_str);

                info!(
                    "update_packages_json. found hash and remote hash: {0} {1}",
                    hash_str, remote_hash_str
                );
                if hash_str != remote_hash_str {
                    info!("update_packages_json. hash does not match. downloading");
                    should_download = true;
                } else {
                    should_download = false;
                }
            }
        }

        if should_download {
            info!("update_packages_json. downloading new {}.json", remote_path);

            let remote_packages_url =
                std::format!("{0}/{1}.json", &config_parsed["host_url"], remote_path);
            let mut download_complete = false;
            let local_packages_temp_path =
                path_to_packages_file().with_file_name(std::format!("{}-temp.json", remote_path));

            match reqwest::blocking::get(&remote_packages_url) {
                Ok(mut response) => {
                    let mut dest = fs::File::create(&local_packages_temp_path)?;
                    io::copy(&mut response, &mut dest)?;
                    download_complete = true;
                }
                Err(err) => {
                    error!("update_packages_json. download err: {:?}", err);
                }
            }

            if download_complete {
                let new_hash_str = generate_hash_from_file_path(&local_packages_temp_path)?;
                if new_hash_str == remote_hash_str {
                    info!("update_packages_json. new downloaded hash matches");
                    fs::rename(local_packages_temp_path, packages_json_file)?;
                } else {
                    info!("update_packages_json. new downloaded hash does not match");
                    fs::remove_file(local_packages_temp_path)?;
                }
            }
        }

        Ok(())
    }

    pub fn get_game_info(app_id: &str) -> Option<json::JsonValue> {
        let packages_json_file = path_to_packages_file();
        let json_str = match fs::read_to_string(packages_json_file) {
            Ok(s) => s,
            Err(err) => {
                info!("read err: {:?}", err);
                return None;
            }
        };
        let parsed = match json::parse(&json_str) {
            Ok(j) => j,
            Err(err) => {
                info!("parsing err: {:?}", err);
                return None;
            }
        };
        let game_info = parsed[app_id].clone();

        match find_user_packages_file() {
            Some(user_packages_file) => {
                info!("{:?}", user_packages_file);

                let user_json_str = match fs::read_to_string(user_packages_file) {
                    Ok(s) => s,
                    Err(err) => {
                        let error_message = std::format!("user-packages.json read err: {:?}", err);
                        error!("{:?}", error_message);
                        /*match show_error("User Packages Error", &error_message) {
                            Ok(s) => s,
                            Err(_err) => {}
                        }*/
                        return None;
                    }
                };

                let user_parsed = match json::parse(&user_json_str) {
                    Ok(j) => j,
                    Err(err) => {
                        let error_message = std::format!("user-packages.json parsing err: {:?}", err);
                        error!("{:?}", error_message);
                        /*match show_error("User Packages Error", &error_message) {
                            Ok(s) => s,
                            Err(_err) => {}
                        }*/
                        return None;
                    }
                };

                let user_game_info = user_parsed[app_id].clone();
                if user_game_info.is_null() {
                    if !user_parsed["default"].is_null()
                        && (game_info.is_null()
                            || (!user_parsed["override_all_with_user_default"].is_null()
                                && user_parsed["override_all_with_user_default"] == true))
                    {
                        info!("game info using user default");
                        return Some(user_parsed["default"].clone());
                    }
                } else {
                    info!("user_packages_file used for game_info");
                    return Some(user_game_info);
                }
            }
            None => {
                info!("user_packages_file not found");
            }
        };

        if game_info.is_null() {
            if !parsed["default"].is_null() {
                info!("game info using default");
                Some(parsed["default"].clone())
            } else {
                None
            }
        } else {
            Some(game_info)
        }
    }

    pub fn get_engines_info() -> Option<(json::JsonValue, json::JsonValue)> {
        let packages_json_file = path_to_packages_file();
        let json_str = match fs::read_to_string(packages_json_file) {
            Ok(s) => s,
            Err(err) => {
                error!("read err: {:?}", err);
                return None;
            }
        };
        let parsed = match json::parse(&json_str) {
            Ok(j) => j,
            Err(err) => {
                error!("parsing err: {:?}", err);
                return None;
            }
        };

        if parsed["engines"].is_null() || parsed["noticeMap"].is_null() {
            None
        } else {
            Some((parsed["engines"].clone(), parsed["noticeMap"].clone()))
        }
    }
