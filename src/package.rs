#![allow(clippy::or_fun_call)]

extern crate reqwest;
extern crate tar;
extern crate xz2;

use bzip2::read::BzDecoder;
use flate2::read::GzDecoder;
use log::{error, info};
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::io::Write;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use tar::Archive;
use xz2::read::XzDecoder;

use crate::client;
use crate::user_env;

extern crate steamlocate;
use steamlocate::SteamDir;

pub static LUX_DISABLE_DEFAULT_CONFIRM: &str = "LUX_DISABLE_DEFAULT_CONFIRM";

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
    panic!("{}", msg);
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
    folder_path.join("packagessniper_v2.json")
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

#[derive(Serialize, Deserialize, Debug)]
pub struct CmdReplacement {
    #[serde(with = "serde_regex")]
    pub match_cmd: Regex,
    pub cmd: String,
    pub args: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PackageMetadata {
    engine_version: String,
    commands: Vec<CmdReplacement>,
}

#[derive(Clone)]
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

pub fn generate_hash_from_file_path(file_path: &Path) -> io::Result<String> {
    let mut file = fs::File::open(file_path)?;
    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher)?;
    let hash_result = hasher.finalize();
    let hash_str = hex::encode(hash_result);
    Ok(hash_str)
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

    let remote_path = "packagessniper_v2";

    let remote_hash_url = std::format!("{0}/{1}.hash256", &config_parsed["host_url"], remote_path);
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

        match reqwest::blocking::get(remote_packages_url) {
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

pub fn convert_notice_to_str(
    notice_item: &json::JsonValue,
    notice_map: &json::JsonValue,
) -> String {
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

pub fn convert_game_info_with_choice(
    choice_name: String,
    game_info: &mut json::JsonValue,
) -> io::Result<()> {
    let mut choice_data = HashMap::new();
    let mut download_array = json::JsonValue::new_array();

    if game_info["choices"].is_null() {
        return Err(Error::new(ErrorKind::Other, "choices array null"));
    }

    for entry in game_info["choices"].members() {
        if entry["name"].is_null() {
            return Err(Error::new(ErrorKind::Other, "missing choice info"));
        }
        choice_data.insert(entry["name"].to_string(), entry.clone());
    }

    if !choice_data.contains_key(&choice_name) {
        return Err(Error::new(
            ErrorKind::Other,
            "choices array does not contain engine choice",
        ));
    }

    for (key, value) in choice_data[&choice_name].entries() {
        if key == "name" || key == "download" {
            continue;
        }
        game_info[key] = value.clone();
    }

    for entry in game_info["download"].members() {
        if choice_data[&choice_name]["download"].is_null()
            || choice_data[&choice_name]["download"].contains(entry["name"].clone())
        {
            match download_array.push(entry.clone()) {
                Ok(()) => {}
                Err(_) => {
                    return Err(Error::new(
                        ErrorKind::Other,
                        "Error pushing to download array",
                    ));
                }
            };
        }
    }

    game_info["download"] = download_array;
    game_info.remove("choices");

    Ok(())
}

pub fn json_to_downloads(
    app_id: &str,
    game_info: &json::JsonValue,
) -> io::Result<Vec<PackageInfo>> {
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

fn unpack_tarball(
    tarball: &Path,
    game_info: &json::JsonValue,
    name: &str,
    sender: &std::sync::mpsc::Sender<String>,
) -> io::Result<()> {
    let package_name = tarball
        .file_name()
        .and_then(|x| x.to_str())
        .and_then(|x| x.split('.').next())
        .ok_or_else(|| Error::new(ErrorKind::Other, "package has no name?"))?;

    let status_obj = client::StatusObj {
        label: None,
        progress: None,
        complete: false,
        log_line: Some(format!("Unpacking {}", package_name)),
        error: None,
        prompt_items: None,
    };
    let status_str = serde_json::to_string(&status_obj).unwrap();
    sender.send(status_str).unwrap();

    let transform = |path: &PathBuf| -> PathBuf {
        match path.as_path().to_str() {
            Some("manifest.json") => PathBuf::from(format!("manifests.lux/{}.json", &package_name)),
            _ => PathBuf::from(path.strip_prefix("dist").unwrap_or(path)),
        }
    };

    info!("installing: {}", package_name);

    let mut extract_location: String = String::new();
    let mut strip_prefix: String = String::new();
    let mut decode_as_zip = false;
    let mut decode_as_7z = false;

    let file_extension = Path::new(&tarball)
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or("");

    if !&game_info["download_config"].is_null()
        && !&game_info["download_config"][&name.to_string()].is_null()
    {
        let file_download_config = &game_info["download_config"][&name.to_string()];
        if !file_download_config["extract_location"].is_null() {
            extract_location = file_download_config["extract_location"].to_string();
            info!(
                "install changing extract location with config {}",
                extract_location
            );
        }
        if !file_download_config["strip_prefix"].is_null() {
            strip_prefix = file_download_config["strip_prefix"].to_string();
            info!("install changing prefix with config {}", strip_prefix);
        }
    }

    if file_extension == "zip" || file_extension == "bin" {
        decode_as_zip = true;
        info!("install changing decoder to zip");
    } else if file_extension == "7z" {
        decode_as_7z = true;
        info!("install changing decoder to 7z");
    }

    let file = fs::File::open(tarball)?;

    if decode_as_zip {
        let mut archive = zip::ZipArchive::new(file).unwrap();
        for i in 0..archive.len() {
            let mut file = archive.by_index(i).unwrap();

            if file.is_dir() {
                continue;
            }

            let mut new_path = PathBuf::from(file.name());

            if !strip_prefix.is_empty() {
                new_path = new_path.strip_prefix(&strip_prefix).unwrap().to_path_buf();
            }

            if !extract_location.is_empty() {
                new_path = Path::new(&extract_location).join(new_path);
            }

            info!("install: {:?}", &new_path);

            if new_path.parent().is_some() {
                fs::create_dir_all(new_path.parent().unwrap())?;
            }

            let _ = fs::remove_file(&new_path);
            let mut outfile = fs::File::create(&new_path).unwrap();
            io::copy(&mut file, &mut outfile).unwrap();
        }
    } else if decode_as_7z {
        sevenz_rust::decompress_with_extract_fn(
            file,
            extract_location,
            |entry, reader, dest| {
                if entry.is_directory() {
                    return Ok(true);
                }

                let mut new_path = PathBuf::from(dest);

                if !strip_prefix.is_empty() {
                    new_path = new_path.strip_prefix(&strip_prefix).unwrap().to_path_buf();
                }

                info!("install: {:?}", &new_path);

                if new_path.parent().is_some() {
                    fs::create_dir_all(new_path.parent().unwrap())?;
                }

                let _ = fs::remove_file(&new_path);
                let mut outfile = fs::File::create(&new_path).unwrap();
                io::copy(reader, &mut outfile).unwrap();
                Ok(true)
            },
        )
        .expect("complete");
    } else {
        let decoder: Box<dyn std::io::Read>;

        if file_extension == "bz2" {
            decoder = Box::new(BzDecoder::new(file));
        } else if file_extension == "gz" {
            decoder = Box::new(GzDecoder::new(file));
        } else if file_extension == "xz" {
            decoder = Box::new(XzDecoder::new(file));
        } else {
            info!("detected copy since file_extension not matching known");
            return copy_only(tarball, sender);
        }

        let mut archive = Archive::new(decoder);

        for entry in archive.entries()? {
            let mut file = entry?;
            let old_path = PathBuf::from(file.header().path()?);
            let mut new_path = transform(&old_path);
            if new_path.to_str().map_or(false, |x| x.is_empty()) {
                continue;
            }

            if !strip_prefix.is_empty() {
                new_path = new_path.strip_prefix(&strip_prefix).unwrap().to_path_buf();
            }

            if !extract_location.is_empty() {
                new_path = Path::new(&extract_location).join(new_path);
            }

            info!("install: {:?}", &new_path);
            if new_path.parent().is_some() {
                fs::create_dir_all(new_path.parent().unwrap())?;
            }
            let _ = fs::remove_file(&new_path);
            file.unpack(&new_path)?;
        }
    }

    let status_obj = client::StatusObj {
        label: None,
        progress: None,
        complete: false,
        log_line: Some(format!("Unpacking complete for {}", package_name)),
        error: None,
        prompt_items: None,
    };
    let status_str = serde_json::to_string(&status_obj).unwrap();
    sender.send(status_str).unwrap();

    Ok(())
}

fn copy_only(path: &Path, sender: &std::sync::mpsc::Sender<String>) -> io::Result<()> {
    let package_name = path
        .file_name()
        .and_then(|x| x.to_str())
        .ok_or_else(|| Error::new(ErrorKind::Other, "package has no name?"))?;

    let status_obj = client::StatusObj {
        label: None,
        progress: Some(0),
        complete: false,
        log_line: Some(format!("Copying {}", package_name)),
        error: None,
        prompt_items: None,
    };
    let status_str = serde_json::to_string(&status_obj).unwrap();
    sender.send(status_str).unwrap();

    info!("copying: {}", package_name);
    fs::copy(path, package_name)?;

    let status_obj_complete = client::StatusObj {
        label: None,
        progress: Some(0),
        complete: false,
        log_line: Some(format!("Copying complete for {}", package_name)),
        error: None,
        prompt_items: None,
    };
    let status_str_complete = serde_json::to_string(&status_obj_complete).unwrap();
    sender.send(status_str_complete).unwrap();

    Ok(())
}

pub fn is_setup_complete(setup_info: &json::JsonValue) -> bool {
    let setup_complete = Path::new(&setup_info["complete_path"].to_string()).exists();
    setup_complete
}

pub fn install(
    game_info: &json::JsonValue,
    sender: &std::sync::mpsc::Sender<String>,
) -> io::Result<()> {
    let app_id = user_env::steam_app_id();

    let packages: std::slice::Iter<'_, json::JsonValue> = game_info["download"].members();

    let status_obj = client::StatusObj {
        label: Some("Installing".to_string()),
        progress: Some(0),
        complete: false,
        log_line: None,
        error: None,
        prompt_items: None,
    };
    let status_str = serde_json::to_string(&status_obj).unwrap();
    sender.send(status_str).unwrap();

    let mut setup_complete = false;
    if !game_info["setup"].is_null() {
        setup_complete = is_setup_complete(&game_info["setup"]);
    }

    let config_json_file = user_env::tool_dir().join("config.json");
    let config_json_str = fs::read_to_string(config_json_file)?;
    let config_parsed = json::parse(&config_json_str).unwrap();
    let mut hash_check_install = false;
    if !config_parsed["hash_check_install"].is_null() {
        let temp_value = &config_parsed["hash_check_install"];
        hash_check_install = temp_value == true;
    }

    for file_info in packages {
        let file = file_info["file"]
            .as_str()
            .ok_or_else(|| Error::new(ErrorKind::Other, "missing info about this game"))?;

        let name = file_info["name"].to_string();
        let mut cache_dir = &app_id;
        if file_info["cache_by_name"] == true {
            cache_dir = &name;
        }

        if setup_complete
            && !&game_info["download_config"].is_null()
            && !&game_info["download_config"][&name.to_string()].is_null()
            && !&game_info["download_config"][&name.to_string()]["setup"].is_null()
            && game_info["download_config"][&name.to_string()]["setup"] == true
        {
            continue;
        }

        if hash_check_install {
            if let Some(install_file_path) = find_cached_file(cache_dir, file) {
                let status_obj = client::StatusObj {
                    label: None,
                    progress: None,
                    complete: false,
                    log_line: Some(format!("Checking install for {}", name)),
                    error: None,
                    prompt_items: None,
                };
                let status_str = serde_json::to_string(&status_obj).unwrap();
                sender.send(status_str).unwrap();

                info!("hash_check_install is enabled, checking for {}", name);
                let hash_file_path = std::format!("{}.hash", name);
                let install_file_hash = generate_hash_from_file_path(&install_file_path)?;

                if let Some(cached_hash_path) = find_cached_file(&app_id, hash_file_path.as_str()) {
                    info!(
                        "{} has been found, checking hash against file",
                        hash_file_path
                    );

                    let cached_hash_value = fs::read_to_string(cached_hash_path)?;
                    info!(
                        "cached hash is {}; install file hash is {}",
                        cached_hash_value, install_file_hash
                    );
                    if cached_hash_value == install_file_hash {
                        info!("hash for {} is same, skipping install", name);

                        let status_obj = client::StatusObj {
                            label: None,
                            progress: None,
                            complete: false,
                            log_line: Some(format!(
                                "Skipping install for {}, as hash is the same",
                                name
                            )),
                            error: None,
                            prompt_items: None,
                        };
                        let status_str = serde_json::to_string(&status_obj).unwrap();
                        sender.send(status_str).unwrap();

                        continue;
                    }
                }

                let hash_dest_path = place_cached_file(&app_id, &hash_file_path).unwrap();
                let mut hash_dest_file = fs::File::create(&hash_dest_path)?;
                hash_dest_file
                    .write_all(install_file_hash.as_bytes())
                    .unwrap();
            }
        }

        match find_cached_file(cache_dir, file) {
            Some(path) => {
                match unpack_tarball(&path, game_info, &name, sender) {
                    Ok(()) => {}
                    Err(err) => {
                        return Err(err);
                    }
                };
            }
            None => {
                return Err(Error::new(ErrorKind::Other, "package file not found"));
            }
        }
    }
    Ok(())
}

pub fn get_game_info(app_id: &str) -> io::Result<json::JsonValue> {
    let packages_json_file = path_to_packages_file();
    let json_str = match fs::read_to_string(packages_json_file) {
        Ok(s) => s,
        Err(err) => {
            info!("read err: {:?}", err);
            return Err(err);
        }
    };
    let parsed = match json::parse(&json_str) {
        Ok(j) => j,
        Err(err) => {
            let error_message = std::format!("parsing err: {:?}", err);
            error!("{}", error_message);
            return Err(Error::new(ErrorKind::Other, error_message));
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
                    return Err(Error::new(ErrorKind::Other, error_message));
                }
            };

            let user_parsed = match json::parse(&user_json_str) {
                Ok(j) => j,
                Err(err) => {
                    let error_message = std::format!("user-packages.json parsing err: {:?}", err);
                    error!("{:?}", error_message);
                    return Err(Error::new(ErrorKind::Other, error_message));
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
                    return Ok(user_parsed["default"].clone());
                }
            } else {
                info!("user_packages_file used for game_info");
                return Ok(user_game_info);
            }
        }
        None => {
            info!("user_packages_file not found");
        }
    };

    if game_info.is_null() {
        if !parsed["default"].is_null() {
            info!("game info using default");
            Ok(parsed["default"].clone())
        } else {
            Err(Error::new(
                ErrorKind::Other,
                "Game info not found & no default",
            ))
        }
    } else {
        Ok(game_info)
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

pub fn get_app_id_deps_paths(deps: &json::JsonValue) -> Option<()> {
    match SteamDir::locate() {
        Some(mut steamdir) => {
            for entry in deps.members() {
                info!("get_app_id_deps_paths. searching for app id {}.", entry);
                let app_id = entry.as_u32()?;

                match steamdir.app(&app_id) {
                    Some(app_location) => {
                        let app_location_path = app_location.path.clone();
                        let app_location_str =
                            &app_location_path.into_os_string().into_string().unwrap();
                        info!(
                            "get_app_id_deps_paths. app id {} found at {:#?}.",
                            app_id, app_location_str
                        );
                        user_env::set_env_var(
                            &std::format!("DEPPATH_{}", app_id).to_string(),
                            app_location_str,
                        );
                    }
                    None => {
                        info!("get_app_id_deps_paths. app id {} not found.", app_id);
                    }
                }
            }

            Some(())
        }
        None => {
            info!("get_app_id_deps_paths. steamdir not found.");
            None
        }
    }
}
