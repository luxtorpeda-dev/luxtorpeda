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

use crate::dialog::default_choice_confirmation_prompt;
use crate::dialog::show_choices;
use crate::dialog::show_error;
use crate::dialog::show_question;
use crate::dialog::start_progress;
use crate::dialog::ProgressState;
use crate::user_env;

extern crate steamlocate;
use steamlocate::SteamDir;

static LUX_DISABLE_DEFAULT_CONFIRM: &str = "LUX_DISABLE_DEFAULT_CONFIRM";

fn place_cached_file(app_id: &str, file: &str) -> io::Result<PathBuf> {
    let xdg_dirs = xdg::BaseDirectories::new().unwrap();
    let path_str = format!("luxtorpeda/{}/{}", app_id, file);
    xdg_dirs.place_cache_file(path_str)
}

fn find_cached_file(app_id: &str, file: &str) -> Option<PathBuf> {
    let xdg_dirs = xdg::BaseDirectories::new().unwrap();
    let path_str = format!("luxtorpeda/{}/{}", app_id, file);
    xdg_dirs.find_cache_file(path_str)
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
    fs::create_dir_all(&folder_path).unwrap();
    folder_path.join("packagesruntime.json")
}

pub fn path_to_cache() -> PathBuf {
    let xdg_dirs = xdg::BaseDirectories::new().unwrap();
    let cache_home = xdg_dirs.get_cache_home();
    let folder_path = cache_home.join("luxtorpeda");
    fs::create_dir_all(&folder_path).unwrap();
    folder_path
}

pub fn path_to_config() -> PathBuf {
    let xdg_dirs = xdg::BaseDirectories::new().unwrap();
    let config_home = xdg_dirs.get_config_home();
    let folder_path = config_home.join("luxtorpeda");
    fs::create_dir_all(&folder_path).unwrap();
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

struct PackageInfo {
    name: String,
    url: String,
    file: String,
    cache_by_name: bool,
}

#[derive(Debug)]
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
    let json_str = fs::read_to_string(file_path)?;
    let mut hasher = Sha1::new();
    hasher.update(json_str);
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

fn convert_notice_to_str(notice_item: &json::JsonValue, notice_map: &json::JsonValue) -> String {
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

fn pick_engine_choice(app_id: &str, game_info: &json::JsonValue) -> io::Result<String> {
    let check_default_choice_file_path = place_config_file(app_id, "default_engine_choice.txt")?;
    if check_default_choice_file_path.exists() {
        info!("show choice. found default choice.");
        let default_engine_choice_str = fs::read_to_string(check_default_choice_file_path)?;
        info!(
            "show choice. found default choice. choice is {:?}",
            default_engine_choice_str
        );

        let mut should_show_confirm = true;

        let config_json_file = user_env::tool_dir().join("config.json");
        let config_json_str = fs::read_to_string(config_json_file)?;
        let config_parsed = json::parse(&config_json_str).unwrap();

        if !config_parsed["disable_default_confirm"].is_null() {
            let disable_default_confirm = &config_parsed["disable_default_confirm"];
            if disable_default_confirm == true {
                info!("show choice. disabling default confirm because of config");
                should_show_confirm = false;
            }
        }

        match env::var(LUX_DISABLE_DEFAULT_CONFIRM) {
            Ok(val) => {
                if val == "1" {
                    info!("show choice. disabling default confirm because of env");
                    should_show_confirm = false;
                } else if val == "0" {
                    info!("show choice. enabling default confirm because of env");
                    should_show_confirm = true;
                }
            }
            Err(err) => {
                info!("LUX_DISABLE_DEFAULT_CONFIRM not found: {}", err);
            }
        }

        let mut use_default = true;
        if should_show_confirm {
            if let Some(()) = default_choice_confirmation_prompt(
                "Default Choice Confirmation",
                &default_engine_choice_str,
            ) {
                use_default = false;

                let config_path = path_to_config();
                let folder_path = config_path.join(&app_id);
                match fs::remove_dir_all(folder_path) {
                    Ok(()) => {
                        info!("clear config done");
                    }
                    Err(err) => {
                        error!("clear config. err: {:?}", err);
                    }
                }
            };
        }

        if use_default {
            return Ok(default_engine_choice_str);
        }
    }

    let engines_option = get_engines_info();

    let mut choices: Vec<ChoiceInfo> = vec![];
    for entry in game_info["choices"].members() {
        if entry["name"].is_null() {
            return Err(Error::new(ErrorKind::Other, "missing choice info"));
        }

        let mut choice_info = ChoiceInfo {
            name: entry["name"].to_string(),
            notices: Vec::new(),
        };

        let mut engine_name = entry["name"].to_string();
        if !entry["engine_name"].is_null() {
            engine_name = entry["engine_name"].to_string();
        }

        let engine_name_clone = engine_name.clone();
        if let Some((ref engines, ref notice_map)) = engines_option {
            if !engines[engine_name_clone].is_null() {
                let engine_name_clone_clone = engine_name.clone();
                let engine_name_clone_clone_two = engine_name.clone();
                let engine_name_clone_clone_three = engine_name.clone();
                let engine_name_clone_clone_four = engine_name.clone();

                if !engines[engine_name_clone_clone]["notices"].is_null() {
                    for entry in engines[engine_name]["notices"].members() {
                        choice_info
                            .notices
                            .push(convert_notice_to_str(entry, notice_map));
                    }
                }

                let controller_not_supported =
                    engines[engine_name_clone_clone_two]["controllerNotSupported"] == true;
                let controller_supported =
                    engines[engine_name_clone_clone_three]["controllerSupported"] == true;
                let controller_supported_manual =
                    engines[engine_name_clone_clone_four]["controllerSupportedManualGame"] == true;

                if controller_not_supported {
                    choice_info
                        .notices
                        .push("Engine Does Not Have Native Controller Support".to_string());
                } else if controller_supported && game_info["controllerSteamDefault"] == true {
                    choice_info.notices.push(
                        "Engine Has Native Controller Support And Works Out of the Box".to_string(),
                    );
                } else if controller_supported_manual && game_info["controllerSteamDefault"] == true
                {
                    choice_info.notices.push(
                        "Engine Has Native Controller Support But Needs Manual In-Game Settings"
                            .to_string(),
                    );
                } else if controller_supported
                    && (game_info["controllerSteamDefault"].is_null()
                        || game_info["controllerSteamDefault"] != true)
                {
                    choice_info.notices.push(
                        "Engine Has Native Controller Support But Needs Manual Steam Settings"
                            .to_string(),
                    );
                }
            }

            if game_info["cloudNotAvailable"] == true {
                choice_info
                    .notices
                    .push("Game Does Not Have Cloud Saves".to_string());
            } else if game_info["cloudAvailable"] == true
                && (game_info["cloudSupported"].is_null() || game_info["cloudSupported"] != true)
            {
                choice_info
                    .notices
                    .push("Game Has Cloud Saves But Unknown Status".to_string());
            } else if game_info["cloudAvailable"] == true && game_info["cloudSupported"] == true {
                choice_info
                    .notices
                    .push("Cloud Saves Supported".to_string());
            } else if game_info["cloudAvailable"] == true && game_info["cloudIssue"] == true {
                choice_info
                    .notices
                    .push("Cloud Saves Not Supported".to_string());
            }

            if !game_info["notices"].is_null() {
                for entry in game_info["notices"].members() {
                    choice_info
                        .notices
                        .push(convert_notice_to_str(entry, notice_map));
                }
            }
        }

        choices.push(choice_info);
    }

    let (choice_name, default_choice) =
        match show_choices("Pick the engine below", "Select Engine", &choices) {
            Ok(s) => s,
            Err(_) => {
                error!("show choice. dialog was rejected");
                return Err(Error::new(ErrorKind::Other, "Show choices failed"));
            }
        };

    info!("engine choice: {:?}", choice_name);

    if !default_choice.is_empty() {
        info!("default engine choice requested for {}", default_choice);
        let default_choice_file_path = place_config_file(app_id, "default_engine_choice.txt")?;
        let mut default_choice_file = File::create(default_choice_file_path)?;
        default_choice_file.write_all(default_choice.as_bytes())?;
    }

    Ok(choice_name)
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

fn json_to_downloads(app_id: &str, game_info: &json::JsonValue) -> io::Result<Vec<PackageInfo>> {
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

pub fn download_all(app_id: String) -> io::Result<String> {
    let mut game_info = get_game_info(app_id.as_str())
        .ok_or_else(|| Error::new(ErrorKind::Other, "missing info about this game"))?;

    let mut engine_choice = String::new();

    if !game_info["choices"].is_null() {
        info!("showing engine choices");

        engine_choice = match pick_engine_choice(app_id.as_str(), &game_info) {
            Ok(s) => s,
            Err(err) => {
                return Err(err);
            }
        };

        match convert_game_info_with_choice(engine_choice.clone(), &mut game_info) {
            Ok(()) => {
                info!("engine choice complete");
            }
            Err(err) => {
                return Err(err);
            }
        };
    }

    if !game_info["app_ids_deps"].is_null() {
        match get_app_id_deps_paths(&game_info["app_ids_deps"]) {
            Some(()) => {
                info!("download_all. get_app_id_deps_paths completed");
            }
            None => {
                info!("download_all. warning: get_app_id_deps_paths not completed");
            }
        }
    }

    if game_info["download"].is_null() {
        info!("skipping downloads (no urls defined for this package)");
        return Ok(engine_choice);
    }

    let downloads = json_to_downloads(app_id.as_str(), &game_info)?;

    if downloads.is_empty() {
        return Ok(engine_choice);
    }

    let mut dialog_message = String::new();

    if !game_info["information"].is_null() && game_info["information"]["non_free"] == true {
        dialog_message = std::format!(
            "This engine uses a non-free engine ({0}). Are you sure you want to continue?",
            game_info["information"]["license"]
        );
    } else if !game_info["information"].is_null()
        && game_info["information"]["closed_source"] == true
    {
        dialog_message = "This engine uses assets from the closed source release. Are you sure you want to continue?".to_string();
    }

    if !dialog_message.is_empty() {
        match show_question("License Warning", &dialog_message) {
            Some(_) => {
                info!("show license warning. dialog was accepted");
            }
            None => {
                info!("show license warning. dialog was rejected");
                return Err(Error::new(ErrorKind::Other, "dialog was rejected"));
            }
        };
    }

    let progress_state = ProgressState {
        interval: 0,
        status: "".to_string(),
        close: false,
        complete: false,
        error: false,
        error_str: "".to_string(),
    };
    let mutex = std::sync::Mutex::new(progress_state);
    let arc = std::sync::Arc::new(mutex);
    let progress_arc = arc.clone();
    let loop_arc = arc.clone();

    let download_thread = std::thread::spawn(move || {
        let client = reqwest::Client::new();

        for (i, info) in downloads.iter().enumerate() {
            let app_id = app_id.to_string();
            info!("starting download on: {} {}", i, info.name.clone());

            let status_arc = loop_arc.clone();
            let mut guard = status_arc.lock().unwrap();
            guard.status = std::format!(
                "Downloading {}/{} - {}",
                i + 1,
                downloads.len(),
                info.name.clone()
            );
            guard.interval = 0;
            std::mem::drop(guard);

            let download_arc = loop_arc.clone();
            let download_err_arc = loop_arc.clone();
            match Runtime::new().unwrap().block_on(download(
                app_id.as_str(),
                info,
                download_arc,
                &client,
            )) {
                Ok(_) => {}
                Err(ref err) => {
                    error!("download of {} error: {}", info.name.clone(), err);
                    let mut guard = download_err_arc.lock().unwrap();
                    guard.close = true;
                    guard.error = true;

                    if err.to_string() != "progress update failed" {
                        guard.error_str =
                            std::format!("Download of {} Error: {}", info.name.clone(), err);
                    }

                    std::mem::drop(guard);

                    let mut cache_dir = app_id;
                    if info.cache_by_name {
                        cache_dir = info.name.clone();
                    }
                    let dest_file = place_cached_file(&cache_dir, &info.file).unwrap();
                    if dest_file.exists() {
                        fs::remove_file(dest_file).unwrap();
                    }
                }
            };

            let error_check_arc = loop_arc.clone();
            let guard = error_check_arc.lock().unwrap();
            if !guard.error {
                info!("completed download on: {} {}", i, info.name.clone());
            } else {
                error!("failed download on: {} {}", i, info.name.clone());
                std::mem::drop(guard);
                break;
            }
            std::mem::drop(guard);
        }

        let mut guard = loop_arc.lock().unwrap();
        guard.close = true;
        if !guard.error {
            guard.complete = true;
        }
        std::mem::drop(guard);
    });

    match start_progress(progress_arc) {
        Ok(()) => {}
        Err(_) => {
            error!("download_all. warning: progress not started");
        }
    };

    let mut guard = arc.lock().unwrap();

    if guard.error {
        download_thread
            .join()
            .expect("The download thread has panicked");
        if !guard.error_str.is_empty() {
            show_error("Download Error", &guard.error_str).unwrap();
        }
        return Err(Error::new(ErrorKind::Other, "Download failed"));
    }

    if !guard.complete {
        guard.close = true;
        std::mem::drop(guard);
        download_thread
            .join()
            .expect("The download thread has panicked");
        return Err(Error::new(
            ErrorKind::Other,
            "Download failed, not complete",
        ));
    }

    std::mem::drop(guard);
    download_thread
        .join()
        .expect("The download thread has panicked");

    Ok(engine_choice)
}

async fn download(
    app_id: &str,
    info: &PackageInfo,
    arc: std::sync::Arc<std::sync::Mutex<ProgressState>>,
    client: &Client,
) -> io::Result<()> {
    let target = info.url.clone() + &info.file;

    let mut cache_dir = app_id;
    if info.cache_by_name {
        cache_dir = &info.name;
    }

    info!("download target: {:?}", target);

    let res = client.get(&target).send().await.or(Err(Error::new(
        ErrorKind::Other,
        format!("Failed to GET from '{}'", &target),
    )))?;

    let total_size = res.content_length().ok_or(Error::new(
        ErrorKind::Other,
        format!("Failed to get content length from '{}'", &target),
    ))?;

    let dest_file = place_cached_file(cache_dir, &info.file)?;
    let mut dest = fs::File::create(dest_file)?;
    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();
    let mut total_percentage: i64 = 0;

    while let Some(item) = stream.next().await {
        let chunk = item.or(Err(Error::new(
            ErrorKind::Other,
            "Error while downloading file",
        )))?;
        dest.write_all(&chunk).or(Err(Error::new(
            ErrorKind::Other,
            "Error while writing to file",
        )))?;

        let new = min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        let percentage = ((downloaded as f64 / total_size as f64) * 100_f64) as i64;

        if percentage != total_percentage {
            info!(
                "download {}%: {} out of {}",
                percentage, downloaded, total_size
            );
            let mut guard = arc.lock().unwrap();
            guard.interval = percentage as usize;

            if guard.close {
                std::mem::drop(guard);
                return Err(Error::new(ErrorKind::Other, "progress update failed"));
            }
            std::mem::drop(guard);
            total_percentage = percentage;
        }
    }

    Ok(())
}

fn unpack_tarball(tarball: &Path, game_info: &json::JsonValue, name: &str) -> io::Result<()> {
    let package_name = tarball
        .file_name()
        .and_then(|x| x.to_str())
        .and_then(|x| x.split('.').next())
        .ok_or_else(|| Error::new(ErrorKind::Other, "package has no name?"))?;

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
        if !file_download_config["decode_as_zip"].is_null()
            && file_download_config["decode_as_zip"] == true
        {
            decode_as_zip = true;
            info!("install changing decoder to zip");
        }
    }

    let file = fs::File::open(&tarball)?;

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
    } else {
        let file_extension = Path::new(&tarball)
            .extension()
            .and_then(OsStr::to_str)
            .unwrap();
        let decoder: Box<dyn std::io::Read>;

        if file_extension == "bz2" {
            decoder = Box::new(BzDecoder::new(file));
        } else if file_extension == "gz" {
            decoder = Box::new(GzDecoder::new(file));
        } else {
            decoder = Box::new(XzDecoder::new(file));
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

    Ok(())
}

fn copy_only(path: &Path) -> io::Result<()> {
    let package_name = path
        .file_name()
        .and_then(|x| x.to_str())
        .ok_or_else(|| Error::new(ErrorKind::Other, "package has no name?"))?;

    info!("copying: {}", package_name);
    fs::copy(path, package_name)?;

    Ok(())
}

pub fn is_setup_complete(setup_info: &json::JsonValue) -> bool {
    let setup_complete = Path::new(&setup_info["complete_path"].to_string()).exists();
    setup_complete
}

pub fn install(game_info: &json::JsonValue) -> io::Result<()> {
    let app_id = user_env::steam_app_id();

    let packages: std::slice::Iter<'_, json::JsonValue> = game_info["download"].members();

    let mut setup_complete = false;
    if !game_info["setup"].is_null() {
        setup_complete = is_setup_complete(&game_info["setup"]);
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

        match find_cached_file(cache_dir, file) {
            Some(path) => {
                if file_info["copy_only"] == true
                    || (!&game_info["download_config"].is_null()
                        && !&game_info["download_config"][&name.to_string()].is_null()
                        && !&game_info["download_config"][&name.to_string()]["copy_only"].is_null()
                        && game_info["download_config"][&name.to_string()]["copy_only"] == true)
                {
                    copy_only(&path)?;
                } else {
                    match unpack_tarball(&path, game_info, &name) {
                        Ok(()) => {}
                        Err(err) => {
                            show_error(
                                "Unpack Error",
                                &std::format!("Error unpacking {}: {}", &file, &err),
                            )?;
                            return Err(err);
                        }
                    };
                }
            }
            None => {
                show_error("Run Error", "Package file not found")?;
                return Err(Error::new(ErrorKind::Other, "package file not found"));
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
                    match show_error("User Packages Error", &error_message) {
                        Ok(s) => s,
                        Err(_err) => {}
                    }
                    return None;
                }
            };

            let user_parsed = match json::parse(&user_json_str) {
                Ok(j) => j,
                Err(err) => {
                    let error_message = std::format!("user-packages.json parsing err: {:?}", err);
                    error!("{:?}", error_message);
                    match show_error("User Packages Error", &error_message) {
                        Ok(s) => s,
                        Err(_err) => {}
                    }
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

pub fn get_game_info_with_json(app_id: &str, parsed: &json::JsonValue) -> Option<json::JsonValue> {
    let game_info = parsed[app_id].clone();

    if let Some(user_packages_file) = find_user_packages_file() {
        let user_json_str = match fs::read_to_string(user_packages_file) {
            Ok(s) => s,
            Err(err) => {
                let error_message = std::format!("user-packages.json read err: {:?}", err);
                error!("{:?}", error_message);
                return None;
            }
        };

        let user_parsed = match json::parse(&user_json_str) {
            Ok(j) => j,
            Err(err) => {
                let error_message = std::format!("user-packages.json parsing err: {:?}", err);
                error!("{:?}", error_message);
                return None;
            }
        };

        let game_info = user_parsed[app_id].clone();
        if game_info.is_null() {
            if !user_parsed["default"].is_null() {
                return Some(user_parsed["default"].clone());
            }
        } else {
            return Some(game_info);
        }
    };

    if game_info.is_null() {
        None
    } else {
        Some(game_info)
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
