use gdnative::prelude::*;

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
use std::sync::mpsc::channel;
use std::time::Duration;

extern crate steamlocate;
use steamlocate::SteamDir;
mod user_env;

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

#[derive(Serialize, Deserialize, Debug)]
struct StatusObj {
    label: std::option::Option<String>,
    progress: std::option::Option<i64>
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

#[derive(NativeClass)]
#[inherit(Node)]
// register_with attribute can be used to specify custom register function for node signals and properties
#[register_with(Self::register_signals)]
pub struct SignalEmitter;

#[methods]
impl SignalEmitter {
    fn register_signals(builder: &ClassBuilder<Self>) {
        builder.signal("choice_picked").done();
    }

    fn new(_owner: &Node) -> Self {
        SignalEmitter
    }
}

#[derive(NativeClass)]
#[inherit(Node)]
pub struct Package
{
    receiver: std::option::Option<std::sync::mpsc::Receiver<String>>
}

#[methods]
impl Package {
    fn new(_base: &Node) -> Self {
        Package { receiver: None }
    }

    #[method]
    fn _ready(&self, #[base] base: TRef<Node>) {
        let app_id = user_env::steam_app_id();
        let env_args: Vec<String> = env::args().collect();
        let args: Vec<&str> = env_args.iter().map(|a| a.as_str()).collect();
        let cmd_args = &args[2..];

        info!("luxtorpeda version: {}", env!("CARGO_PKG_VERSION"));
        info!("steam_app_id: {:?}", &app_id);
        info!("original command: {:?}", args);
        info!("working dir: {:?}", env::current_dir());
        info!("tool dir: {:?}", user_env::tool_dir());

        if args.len() < 2 {
            //usage();
            std::process::exit(0)
        }

        let exe = cmd_args[0].to_lowercase();
        let exe_args = &cmd_args[1..];

        if exe.ends_with("iscriptevaluator.exe") {
            std::process::exit(0)
        }

        user_env::assure_xdg_runtime_dir();
        user_env::assure_tool_dir(args[0]);

        let emitter = &mut base.get_node("SignalEmitter").unwrap();
        let emitter = unsafe { emitter.assume_safe() };

        emitter
            .connect(
                "choice_picked",
                base,
                "choice_picked",
                VariantArray::new_shared(),
                0,
            )
            .unwrap();

        self.update_packages_json();
        self.ask_for_engine_choice(app_id.as_str(), &base);
    }

    #[method]
    fn _physics_process(&self, #[base] base: TRef<Node>, delta: f64) {
        if let Some(receiver) = &self.receiver {
            if let Ok(new_data) = receiver.try_recv() {
                let emitter = &mut base.get_node("Container/Progress").unwrap();
                let emitter = unsafe { emitter.assume_safe() };
                emitter.emit_signal("progress_change", &[Variant::new(new_data)]);
            }
        }
    }

    fn ask_for_engine_choice(&self, app_id: &str, owner: &Node) -> io::Result<String> {
        let mut game_info = Package::get_game_info(app_id)
            .ok_or_else(|| Error::new(ErrorKind::Other, "missing info about this game"))?;


        if game_info.is_null() {
            return Err(Error::new(ErrorKind::Other, "Unknown app_id"));
        }

        if !game_info["choices"].is_null() {
            let engines_option = Package::get_engines_info();

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

            let choices_str = serde_json::to_string(&choices).unwrap();
            let emitter = &mut owner.get_node("Container/Choices").unwrap();
            let emitter = unsafe { emitter.assume_safe() };
            emitter.emit_signal("choices_found", &[Variant::new(choices_str)]);
        }

        Ok("test".to_string())
    }

    #[method]
    fn choice_picked(&mut self, #[base] owner: &Node, data: Variant) {

        info!("ASD*&&*&*&*&*");
        let msg = format!(
            "Received signal \"choice_picked\" with data {}",
            data.try_to::<String>().unwrap()
        );

        info!("ASDASDASD {}", msg);

        let app_id = user_env::steam_app_id();
        let mut game_info = Self::get_game_info(app_id.as_str()).unwrap();

        let mut engine_choice = data.try_to::<String>().unwrap();

        /*if !game_info["app_ids_deps"].is_null() {
            match get_app_id_deps_paths(&game_info["app_ids_deps"]) {
                Some(()) => {
                    info!("download_all. get_app_id_deps_paths completed");
                }
                None => {
                    info!("download_all. warning: get_app_id_deps_paths not completed");
                }
            }
        }*/

        if game_info["download"].is_null() {
            info!("skipping downloads (no urls defined for this package)");
            return;
        }

        let downloads = json_to_downloads(app_id.as_str(), &game_info).unwrap();

        if downloads.is_empty() {
            return;
        }

        /*let mut dialog_message = String::new();

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
        }*/

        let (sender, receiver) = channel();

        let download_thread = std::thread::spawn(move || {
            let client = reqwest::Client::new();

            for (i, info) in downloads.iter().enumerate() {
                let app_id = app_id.to_string();
                info!("starting download on: {} {}", i, info.name.clone());

                let label_str = std::format!(
                    "Downloading {}/{} - {}",
                    i + 1,
                    downloads.len(),
                    info.name.clone()
                );

                let status_obj = StatusObj { label: Some(label_str), progress: None };
                let status_str = serde_json::to_string(&status_obj).unwrap();
                sender.send(status_str).unwrap();

                match Runtime::new().unwrap().block_on(Self::download(
                    app_id.as_str(),
                    info,
                    sender.clone(),
                    &client,
                )) {
                    Ok(_) => {}
                    Err(ref err) => {
                        error!("download of {} error: {}", info.name.clone(), err);
                       /* let mut guard = download_err_arc.lock().unwrap();
                        guard.close = true;
                        guard.error = true;

                        if err.to_string() != "progress update failed" {
                            guard.error_str =
                                std::format!("Download of {} Error: {}", info.name.clone(), err);
                        }

                        std::mem::drop(guard);*/

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

                /*let error_check_arc = loop_arc.clone();
                let guard = error_check_arc.lock().unwrap();
                if !guard.error {
                    info!("completed download on: {} {}", i, info.name.clone());
                } else {
                    error!("failed download on: {} {}", i, info.name.clone());
                    std::mem::drop(guard);
                    break;
                }
                std::mem::drop(guard);
                }*/
            }
        });

        self.receiver = Some(receiver);

        /*download_thread
            .join();*/
    }

    async fn download(
        app_id: &str,
        info: &PackageInfo,
        sender: std::sync::mpsc::Sender<String>,
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

                let status_obj = StatusObj { label: None, progress: Some(percentage) };
                let status_str = serde_json::to_string(&status_obj).unwrap();
                sender.send(status_str).unwrap();

                total_percentage = percentage;
            }
        }

        Ok(())
    }

    fn update_packages_json(&self) -> io::Result<()> {
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

    fn get_game_info(app_id: &str) -> Option<json::JsonValue> {
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

    fn get_engines_info() -> Option<(json::JsonValue, json::JsonValue)> {
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
}
