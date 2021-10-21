extern crate reqwest;
extern crate tar;
extern crate xz2;

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::ffi::OsStr;
use tar::Archive;
use xz2::read::XzDecoder;
use bzip2::read::BzDecoder;
use flate2::read::GzDecoder;
use sha1::{Sha1, Digest};
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use reqwest::Client;
use futures_util::StreamExt;
use tokio::runtime::Runtime;
use std::cmp::min;
use tokio;

use crate::user_env;

use crate::dialog::show_error;
use crate::dialog::show_choices;
use crate::dialog::show_question;
use crate::dialog::start_progress;
use crate::dialog::ProgressState;

extern crate steamlocate;
use steamlocate::SteamDir;

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

fn path_to_packages_file() -> PathBuf {
    let xdg_dirs = xdg::BaseDirectories::new().unwrap();
    let config_home = xdg_dirs.get_cache_home();
    let folder_path = config_home.join("luxtorpeda");
    fs::create_dir_all(&folder_path).unwrap();
    let path = folder_path.join("packagesruntime.json");
    return path;
}

pub fn path_to_cache() -> PathBuf {
    let xdg_dirs = xdg::BaseDirectories::new().unwrap();
    let config_home = xdg_dirs.get_cache_home();
    let folder_path = config_home.join("luxtorpeda");
    fs::create_dir_all(&folder_path).unwrap();
    return folder_path;
}

pub fn find_user_packages_file() -> Option<PathBuf> {
    let xdg_dirs = xdg::BaseDirectories::new().unwrap();
    let path_str = format!("luxtorpeda/user-packages.json");
    xdg_dirs.find_config_file(path_str)
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
    cache_by_name: bool
}

pub fn get_remote_packages_hash(remote_hash_url: &String) -> Option<String> {
    let remote_hash_response = match reqwest::blocking::get(remote_hash_url) {
        Ok(s) => s,
        Err(err) => {
            println!("get_remote_packages_hash error in get: {:?}", err);
            return None;
        }
    };
    
    let remote_hash_str = match remote_hash_response.text() {
        Ok(s) => s,
        Err(err) => {
            println!("get_remote_packages_hash error in text: {:?}", err);
            return None;
        }
    };
    
    return Some(remote_hash_str);
}

pub fn generate_hash_from_file_path(file_path: &std::path::PathBuf) -> io::Result<String> {
    let json_str = fs::read_to_string(file_path)?;
    let mut hasher = Sha1::new();
    hasher.update(json_str);
    let hash_result = hasher.finalize();
    let hash_str = hex::encode(hash_result);
    return Ok(hash_str);
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
        },
        None => {
            println!("update_packages_json in get_remote_packages_hash call. received none");
            should_download = false;
        }
    }
    
    if should_download {
        if !Path::new(&packages_json_file).exists() {
            should_download = true;
            println!("update_packages_json. {:?} does not exist", packages_json_file);
        } else {
            let hash_str = generate_hash_from_file_path(&packages_json_file)?;
            println!("update_packages_json. found hash: {}", hash_str);
            
            println!("update_packages_json. found hash and remote hash: {0} {1}", hash_str, remote_hash_str);
            if hash_str != remote_hash_str {
                println!("update_packages_json. hash does not match. downloading");
                should_download = true;
            } else {
                should_download = false;
            }
        }
    }
    
    if should_download {
        println!("update_packages_json. downloading new {}.json", remote_path);
        
        let remote_packages_url = std::format!("{0}/{1}.json", &config_parsed["host_url"], remote_path);
        let mut download_complete = false;
        let local_packages_temp_path = path_to_packages_file().with_file_name(std::format!("{}-temp.json", remote_path));
        
        match reqwest::blocking::get(&remote_packages_url) {
            Ok(mut response) => {
                let mut dest = fs::File::create(&local_packages_temp_path)?;
                io::copy(&mut response, &mut dest)?;
                download_complete = true;
            }
            Err(err) => {
                println!("update_packages_json. download err: {:?}", err);
            }
        }
        
        if download_complete {
            let new_hash_str = generate_hash_from_file_path(&local_packages_temp_path)?;
            if new_hash_str == remote_hash_str {
                println!("update_packages_json. new downloaded hash matches");
                fs::rename(local_packages_temp_path, packages_json_file)?;
            } else {
                println!("update_packages_json. new downloaded hash does not match");
                fs::remove_file(local_packages_temp_path)?;
            }
        }
    }
    
    Ok(())
}

fn pick_engine_choice(app_id: &str, game_info: &json::JsonValue) -> io::Result<String> {
    let check_default_choice_file_path = place_config_file(&app_id, "default_engine_choice.txt")?;
    if check_default_choice_file_path.exists() {
        println!("show choice. found default choice.");
        let default_engine_choice_str = fs::read_to_string(check_default_choice_file_path)?;
        println!("show choice. found default choice. choice is {:?}", default_engine_choice_str);

        return Ok(default_engine_choice_str)
    }
        
    let mut choices: Vec<String> = vec![];
    for entry in game_info["choices"].members() {
        if entry["name"].is_null() {
            return Err(Error::new(ErrorKind::Other, "missing choice info"));
        }
        choices.push(entry["name"].to_string());
    }
    
    let (choice_name, default) = match show_choices("Pick the engine below", "Name", &choices) {
        Ok(s) => s,
        Err(_) => {
            println!("show choice. dialog was rejected");
            return Err(Error::new(ErrorKind::Other, "Show choices failed"));
        }
    };
    
    println!("engine choice: {:?}", choice_name);

    if default {
        println!("default engine choice requested");
        let default_choice_file_path = place_config_file(&app_id, "default_engine_choice.txt")?;
        let mut default_choice_file = File::create(default_choice_file_path)?;
        default_choice_file.write_all(choice_name.as_bytes())?;
    }
    
    return Ok(choice_name);
}

pub fn convert_game_info_with_choice(choice_name: String, game_info: &mut json::JsonValue) -> io::Result<()> {
    let mut choice_data = HashMap::new();
    let mut download_array = json::JsonValue::new_array();
    
    if game_info["choices"].is_null() {
        return Err(Error::new(ErrorKind::Other, "choices array null"));
    }
    
    for entry in game_info["choices"].members() {
        if entry["name"].is_null() {
            return Err(Error::new(ErrorKind::Other, "missing choice info"));
        }
        choice_data.insert(
            entry["name"].to_string(),
            entry.clone()
        );
    }
    
    if !choice_data.contains_key(&choice_name) {
        return Err(Error::new(ErrorKind::Other, "choices array does not contain engine choice"));
    }
    
    for (key, value) in choice_data[&choice_name].entries() {
        if key == "name" || key == "download" {
            continue;
        }
        game_info[key] = value.clone();
    }
    
    for entry in game_info["download"].members() {
        if choice_data[&choice_name]["download"].is_null() || choice_data[&choice_name]["download"].contains(entry["name"].clone()) {
            match download_array.push(entry.clone()) {
                Ok(()) => {},
                Err(_) => {
                    return Err(Error::new(ErrorKind::Other, "Error pushing to download array"));
                }
            };
        }
    }
    
    game_info["download"] = download_array;
    game_info.remove("choices");
    
    return Ok(());
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
            println!("{} found in cache (skip)", file);
            continue;
        }

        downloads.push(PackageInfo { name, url, file, cache_by_name });
    }
    Ok(downloads)
}

pub fn download_all(app_id: String) -> io::Result<String> {
    let mut game_info = get_game_info(app_id.as_str())
        .ok_or_else(|| Error::new(ErrorKind::Other, "missing info about this game"))?;

    let mut engine_choice = String::new();

    if !game_info["choices"].is_null() {
        println!("showing engine choices");

        engine_choice = match pick_engine_choice(app_id.as_str(), &game_info) {
            Ok(s) => s,
            Err(err) => {
                return Err(err);
            }
        };

        match convert_game_info_with_choice(engine_choice.clone(), &mut game_info) {
            Ok(()) => {
                println!("engine choice complete");
            },
            Err(err) => {
                return Err(err);
            }
        };
    }

    if !game_info["app_ids_deps"].is_null() {
         match get_app_id_deps_paths(&game_info["app_ids_deps"]) {
            Some(()) => {
                println!("download_all. get_app_id_deps_paths completed");
            },
            None => {
                println!("download_all. warning: get_app_id_deps_paths not completed");
            }
        }
    }

    if game_info["download"].is_null() {
        println!("skipping downloads (no urls defined for this package)");
        return Ok(engine_choice);
    }

    let downloads = json_to_downloads(app_id.as_str(), &game_info)?;

    if downloads.is_empty() {
        return Ok(engine_choice);
    }

    let mut dialog_message = String::new();

    if !game_info["information"].is_null() && game_info["information"]["non_free"] == true {
        dialog_message = std::format!("This engine uses a non-free engine ({0}). Are you sure you want to continue?", game_info["information"]["license"]);
    }
    else if !game_info["information"].is_null() && game_info["information"]["closed_source"] == true {
        dialog_message = "This engine uses assets from the closed source release. Are you sure you want to continue?".to_string();
    }

    if !dialog_message.is_empty() {
        match show_question("License Warning", &dialog_message.to_string()) {
            Some(_) => {
                println!("show license warning. dialog was accepted");
            },
            None => {
                println!("show license warning. dialog was rejected");
                return Err(Error::new(ErrorKind::Other, "dialog was rejected"));
            }
        };
    }

    let progress_state = ProgressState{interval: 0, status: "".to_string(), close: false, complete: false, error: false, error_str: "".to_string()};
    let mutex = std::sync::Mutex::new(progress_state);
    let arc = std::sync::Arc::new(mutex);
    let progress_arc = arc.clone();
    let loop_arc = arc.clone();

    let download_thread = std::thread::spawn(move || {
        let client = reqwest::Client::new();

        for (i, info) in downloads.iter().enumerate() {
            let app_id = app_id.to_string();
            println!("starting download on: {} {}", i, info.name.clone());

            let status_arc = loop_arc.clone();
            let mut guard = status_arc.lock().unwrap();
            guard.status = std::format!("Downloading {}/{} - {}", i+1, downloads.len(), info.name.clone());
            guard.interval = 0;
            std::mem::drop(guard);

            let download_arc = loop_arc.clone();
            let download_err_arc = loop_arc.clone();
            match Runtime::new().unwrap().block_on(download(app_id.as_str(), info, download_arc, &client)) {
                Ok(_) => {},
                Err(ref err) => {
                    println!("download of {} error: {}",info.name.clone(), err);
                    let mut guard = download_err_arc.lock().unwrap();
                    guard.close = true;
                    guard.error = true;

                    if err.to_string() != "progress update failed" {
                        guard.error_str = std::format!("Download of {} Error: {}",info.name.clone(), err);
                    }

                    std::mem::drop(guard);

                    let mut cache_dir = app_id;
                    if info.cache_by_name == true {
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
                println!("completed download on: {} {}", i, info.name.clone());
            } else {
                println!("failed download on: {} {}", i, info.name.clone());
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
        Ok(()) => {},
        Err(_) => {
            println!("download_all. warning: progress not started");
        }
    };

    let check_arc = arc.clone();
    let mut guard = check_arc.lock().unwrap();

    if guard.error {
        download_thread.join().expect("The download thread has panicked");
        if guard.error_str != "" {
            show_error(&"Download Error".to_string(), &guard.error_str).unwrap();
        }
        return Err(Error::new(ErrorKind::Other, "Download failed"));
    }

    if !guard.complete {
        guard.close = true;
        std::mem::drop(guard);
        download_thread.join().expect("The download thread has panicked");
        return Err(Error::new(ErrorKind::Other, "Download failed, not complete"));
    }

    std::mem::drop(guard);
    download_thread.join().expect("The download thread has panicked");

    return Ok(engine_choice);
}

async fn download(app_id: &str, info: &PackageInfo, arc: std::sync::Arc<std::sync::Mutex<ProgressState>>, client: &Client) -> io::Result<()> {
    let target = info.url.clone() + &info.file;

    let mut cache_dir = app_id;
    if info.cache_by_name == true {
        cache_dir = &info.name;
    }

    println!("download target: {:?}", target);

    let res = client
        .get(&target)
        .send()
        .await
        .or(Err(Error::new(ErrorKind::Other, format!("Failed to GET from '{}'", &target))))?;

    let total_size = res
        .content_length()
        .ok_or(Error::new(ErrorKind::Other, format!("Failed to get content length from '{}'", &target)))?;

    let dest_file = place_cached_file(&cache_dir, &info.file)?;
    let mut dest = fs::File::create(dest_file)?;
    let mut downloaded: u64 = 0;
    let mut stream = res.bytes_stream();
    let mut total_percentage: i64 = 0;

    while let Some(item) = stream.next().await {
        let chunk = item.or(Err(Error::new(ErrorKind::Other, format!("Error while downloading file"))))?;
        dest.write(&chunk)
            .or(Err(Error::new(ErrorKind::Other, format!("Error while writing to file"))))?;

        let new = min(downloaded + (chunk.len() as u64), total_size);
        downloaded = new;
        let percentage = ((downloaded as f64 / total_size as f64) * 100 as f64) as i64;

        if percentage != total_percentage {
            println!("download {}%: {} out of {}", percentage, downloaded, total_size);
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

fn unpack_tarball(tarball: &PathBuf, game_info: &json::JsonValue, name: &str) -> io::Result<()> {
    let package_name = tarball
        .file_name()
        .and_then(|x| x.to_str())
        .and_then(|x| x.split('.').next())
        .ok_or_else(|| Error::new(ErrorKind::Other, "package has no name?"))?;

    let transform = |path: &PathBuf| -> PathBuf {
        match path.as_path().to_str() {
            Some("manifest.json") => PathBuf::from(format!("manifests.lux/{}.json", &package_name)),
            _ => PathBuf::from(path.strip_prefix("dist").unwrap_or(&path)),
        }
    };

    eprintln!("installing: {}", package_name);
    
    let mut extract_location: String = String::new();
    let mut strip_prefix: String = String::new();
    let mut decode_as_zip = false;
    
    if !&game_info["download_config"].is_null() && !&game_info["download_config"][&name.to_string()].is_null() {
        let file_download_config = &game_info["download_config"][&name.to_string()];
        if !file_download_config["extract_location"].is_null() {
            extract_location = file_download_config["extract_location"].to_string();
            println!("install changing extract location with config {}", extract_location);
        }
        if !file_download_config["strip_prefix"].is_null() {
            strip_prefix = file_download_config["strip_prefix"].to_string();
            println!("install changing prefix with config {}", strip_prefix);
        }
        if !file_download_config["decode_as_zip"].is_null() && file_download_config["decode_as_zip"] == true {
            decode_as_zip = true;
            println!("install changing decoder to zip");
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
            
            println!("install: {:?}", &new_path);
            
            if new_path.parent().is_some() {
                fs::create_dir_all(new_path.parent().unwrap())?;
            }
            
            let _ = fs::remove_file(&new_path);
            let mut outfile = fs::File::create(&new_path).unwrap();
            io::copy(&mut file, &mut outfile).unwrap();
        }
    } else {
        let file_extension = Path::new(&tarball).extension().and_then(OsStr::to_str).unwrap();
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
            
            println!("install: {:?}", &new_path);
            if new_path.parent().is_some() {
                fs::create_dir_all(new_path.parent().unwrap())?;
            }
            let _ = fs::remove_file(&new_path);
            file.unpack(&new_path)?;
        }
    }

    Ok(())
}

fn copy_only(path: &PathBuf) -> io::Result<()> {
    let package_name = path
        .file_name()
        .and_then(|x| x.to_str())
        .ok_or_else(|| Error::new(ErrorKind::Other, "package has no name?"))?;

    eprintln!("copying: {}", package_name);
    fs::copy(path, package_name)?;

    Ok(())
}

pub fn is_setup_complete(setup_info: &json::JsonValue) -> bool {
    let setup_complete = Path::new(&setup_info["complete_path"].to_string()).exists();
    return setup_complete;
}

pub fn install(game_info: &json::JsonValue) -> io::Result<()> {
    let app_id = user_env::steam_app_id();

    let packages: std::slice::Iter<'_, json::JsonValue> = game_info["download"]
        .members();
        
    let mut setup_complete = false;
    if !game_info["setup"].is_null() {
        setup_complete = is_setup_complete(&game_info["setup"]);
    }

    for file_info in packages {
        let file = file_info["file"].as_str()
            .ok_or_else(|| Error::new(ErrorKind::Other, "missing info about this game"))?;
            
        let name = file_info["name"].to_string();
        let mut cache_dir = &app_id;
        if file_info["cache_by_name"] == true {
            cache_dir = &name;
        }
        
        if setup_complete && !&game_info["download_config"].is_null() && !&game_info["download_config"][&name.to_string()].is_null() && !&game_info["download_config"][&name.to_string()]["setup"].is_null() && game_info["download_config"][&name.to_string()]["setup"] == true {
            continue;
        }
        
        match find_cached_file(&cache_dir, &file) {
            Some(path) => {
                if file_info["copy_only"] == true {
                    copy_only(&path)?;
                }
                else if
                    !&game_info["download_config"].is_null() &&
                    !&game_info["download_config"][&name.to_string()].is_null() &&
                    !&game_info["download_config"][&name.to_string()]["copy_only"].is_null() &&
                    game_info["download_config"][&name.to_string()]["copy_only"] == true {
                    copy_only(&path)?;
                }
                else {
                    match unpack_tarball(&path, &game_info, &name) {
                        Ok(()) => {},
                        Err(err) => {
                            show_error(&"Unpack Error".to_string(), &std::format!("Error unpacking {}: {}", &file, &err).to_string())?;
                            return Err(err);
                        }
                    };
                }
            }
            None => {
                show_error(&"Run Error".to_string(), &"Package file not found".to_string())?;
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
            println!("read err: {:?}", err);
            return None;
        }
    };
    let parsed = match json::parse(&json_str) {
        Ok(j) => j,
        Err(err) => {
            println!("parsing err: {:?}", err);
            return None;
        }
    };
    let game_info = parsed[app_id].clone();
    
    match find_user_packages_file() {
        Some(user_packages_file) => {
            println!("{:?}", user_packages_file);
            
            let user_json_str = match fs::read_to_string(user_packages_file) {
                Ok(s) => s,
                Err(err) => {
                    let error_message = std::format!("user-packages.json read err: {:?}", err);
                    println!("{:?}", error_message);
                    match show_error(&"User Packages Error".to_string(), &error_message) {
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
                    println!("{:?}", error_message);
                    match show_error(&"User Packages Error".to_string(), &error_message) {
                        Ok(s) => s,
                        Err(_err) => {}
                    }
                    return None;
                }
            };
            
            let game_info = user_parsed[app_id].clone();
            if game_info.is_null() {
                if !user_parsed["default"].is_null() {
                    println!("game info using user default");
                    return Some(user_parsed["default"].clone());
                }
            } else {
                println!("user_packages_file used for game_info");
                return Some(game_info)
            }
        },
        None => {
            println!("user_packages_file not found");
        }
    };
    
    if game_info.is_null() {
        if !parsed["default"].is_null() {
            println!("game info using default");
            return Some(parsed["default"].clone());
        }
        else {
            None
        }
    } else {
        Some(game_info)
    }
}

pub fn get_app_id_deps_paths(deps: &json::JsonValue) -> Option<()> {
    match SteamDir::locate() {
        Some(mut steamdir) => {
            for entry in deps.members() {
                println!("get_app_id_deps_paths. searching for app id {}.", entry);
                let app_id = entry.as_u32()?;

                match steamdir.app(&app_id) {
                    Some(app_location) => {
                        let app_location_path = app_location.path.clone();
                        let app_location_str = &app_location_path.into_os_string().into_string().unwrap();
                        println!("get_app_id_deps_paths. app id {} found at {:#?}.", app_id, app_location_str);
                        user_env::set_env_var(&std::format!("DEPPATH_{}", app_id).to_string(), &app_location_str);
                    },
                    None => {
                        println!("get_app_id_deps_paths. app id {} not found.", app_id);
                    }
                }
            }

            return Some(())
        },
        None => {
            println!("get_app_id_deps_paths. steamdir not found.");
            return None
        }
    }
}
