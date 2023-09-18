extern crate reqwest;
extern crate tar;
extern crate xz2;

use ar::Archive as ArArchive;
use bzip2::read::BzDecoder;
use flate2::read::GzDecoder;
use log::{error, info, warn};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::io;
use std::io::Write;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;
use tar::Archive;
use xz2::read::XzDecoder;

use crate::client;
use crate::command::find_game_command;
use crate::config;
use crate::package_metadata;
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
pub fn create_dir_or_show_error(path: &impl AsRef<Path>) {
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

pub fn generate_hash_from_file_path(file_path: &Path) -> io::Result<String> {
    let mut file = fs::File::open(file_path)?;
    let mut hasher = Sha256::new();
    io::copy(&mut file, &mut hasher)?;
    let hash_result = hasher.finalize();
    let hash_str = hex::encode(hash_result);
    Ok(hash_str)
}

pub fn generate_hash_from_string(hashable_string: &String) -> io::Result<String> {
    let mut hasher = Sha256::new();
    hasher.update(hashable_string);
    let hash_result = hasher.finalize();
    let hash_str = hex::encode(hash_result);
    Ok(hash_str)
}

pub fn convert_game_info_with_choice(
    choice_name: String,
    game_info: &mut package_metadata::Game,
) -> io::Result<()> {
    let mut choice_data = HashMap::new();
    let mut new_downloads: Vec<package_metadata::DownloadItem> = vec![];

    if let Some(choices) = game_info.choices.clone() {
        for entry in choices {
            choice_data.insert(entry.name.clone(), entry.clone());
        }
    } else {
        return Err(Error::new(ErrorKind::Other, "choices array null"));
    }

    if !choice_data.contains_key(&choice_name) {
        return Err(Error::new(
            ErrorKind::Other,
            "choices array does not contain engine choice",
        ));
    }

    let engine_choice_data = &choice_data[&choice_name];

    for entry in &game_info.download {
        let mut should_push_download = true;
        if let Some(choice_download) = &engine_choice_data.download {
            if !choice_download.contains(&entry.name) {
                should_push_download = false;
            }
        }

        if should_push_download {
            new_downloads.push(entry.clone());
        }
    }

    game_info.download = new_downloads;
    game_info.update_from_choice(engine_choice_data);
    game_info.choices = None;

    Ok(())
}

pub fn json_to_downloads(
    app_id: &str,
    game_info: &package_metadata::Game,
) -> io::Result<Vec<package_metadata::DownloadItem>> {
    let mut downloads: Vec<package_metadata::DownloadItem> = Vec::new();
    for entry in &game_info.download {
        if entry.name.is_empty() || entry.url.is_empty() || entry.file.is_empty() {
            return Err(Error::new(ErrorKind::Other, "missing download info"));
        }

        let mut cache_dir = app_id;
        if entry.cache_by_name {
            cache_dir = &entry.name;
        }

        if find_cached_file(cache_dir, entry.file.as_str()).is_some() {
            info!("{} found in cache (skip)", entry.file);
            continue;
        }

        downloads.push(entry.clone());
    }
    Ok(downloads)
}

fn unpack_tarball(
    tarball: &Path,
    game_info: &package_metadata::Game,
    name: &str,
    sender: &std::sync::mpsc::Sender<String>,
) -> io::Result<()> {
    let package_name = tarball
        .file_name()
        .and_then(|x| x.to_str())
        .and_then(|x| x.split('.').next())
        .ok_or_else(|| Error::new(ErrorKind::Other, "package has no name?"))?;

    let status_obj = client::StatusObj {
        log_line: Some(format!("Unpacking {}", package_name)),
        ..Default::default()
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
    let mut decode_with_ar = false;

    let file_extension = Path::new(&tarball)
        .extension()
        .and_then(OsStr::to_str)
        .unwrap_or("");

    if let Some(file_download_config) = game_info.find_download_config_by_name(name) {
        if let Some(tmp_extract_location) = file_download_config.extract_location {
            extract_location = tmp_extract_location;
            info!(
                "install changing extract location with config {}",
                extract_location
            );
        }
        if let Some(tmp_strip_prefix) = file_download_config.strip_prefix {
            strip_prefix = tmp_strip_prefix;
            info!("install changing prefix with config {}", strip_prefix);
        }
    }

    if file_extension == "zip" || file_extension == "bin" {
        decode_as_zip = true;
        info!("install changing decoder to zip");
    } else if file_extension == "7z" {
        decode_as_7z = true;
        info!("install changing decoder to 7z");
    } else if file_extension == "deb" {
        decode_with_ar = true;
        info!("install changing decoder to ar");
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
        sevenz_rust::decompress_with_extract_fn(file, extract_location, |entry, reader, dest| {
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
        })
        .expect("complete");
    } else if decode_with_ar {
        let mut archive = ArArchive::new(file);
        while let Some(entry_result) = archive.next_entry() {
            let mut entry = entry_result.unwrap();
            let filename = std::str::from_utf8(entry.header().identifier()).unwrap();
            let new_name = format!("{}_{}", name, filename);
            if filename == "data.tar.xz" {
                let mut new_path = PathBuf::from(filename);

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
                io::copy(&mut entry, &mut outfile).unwrap();

                info!("sending install for {}", new_name);
                match unpack_tarball(&new_path, game_info, &new_name, sender) {
                    Ok(()) => {}
                    Err(err) => {
                        error!("Error on unpack_tarball: {:?}", err)
                    }
                };
            } else {
                info!("skipping install from ar for {}", filename);
            }
        }
    } else {
        let decoder: Box<dyn std::io::Read>;

        if file_extension == "bz2" {
            decoder = Box::new(BzDecoder::new(file));
        } else if file_extension == "gz" || file_extension == "tgz" {
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
        log_line: Some(format!("Unpacking complete for {}", package_name)),
        ..Default::default()
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
        progress: Some(0),
        log_line: Some(format!("Copying {}", package_name)),
        ..Default::default()
    };
    let status_str = serde_json::to_string(&status_obj).unwrap();
    sender.send(status_str).unwrap();

    info!("copying: {}", package_name);
    fs::copy(path, package_name)?;

    let status_obj_complete = client::StatusObj {
        progress: Some(0),

        log_line: Some(format!("Copying complete for {}", package_name)),
        ..Default::default()
    };
    let status_str_complete = serde_json::to_string(&status_obj_complete).unwrap();
    sender.send(status_str_complete).unwrap();

    Ok(())
}

pub fn is_setup_complete(setup_info: &package_metadata::Setup) -> bool {
    let setup_complete = Path::new(&setup_info.complete_path).exists();
    setup_complete
}

pub fn install(
    game_info: &package_metadata::Game,
    sender: &std::sync::mpsc::Sender<String>,
) -> io::Result<()> {
    let app_id = user_env::steam_app_id();

    let status_obj = client::StatusObj {
        label: Some("Installing".to_string()),
        progress: Some(0),
        ..Default::default()
    };
    let status_str = serde_json::to_string(&status_obj).unwrap();
    sender.send(status_str).unwrap();

    let mut setup_complete = false;
    if let Some(setup) = &game_info.setup {
        setup_complete = is_setup_complete(setup);
    }

    let config = config::Config::from_config_file();
    let hash_check_install = config.hash_check_install;

    let mut game_command_file_found = false;
    if let Some((cmd, _)) = find_game_command(game_info, &[]) {
        let cmd_path = Path::new(&cmd);
        if cmd_path.exists() {
            game_command_file_found = true;
        }
    }

    for file_info in &game_info.download {
        let file = &file_info.file;
        let name = &file_info.name;
        let mut cache_dir = &app_id;
        if file_info.cache_by_name {
            cache_dir = name;
        }

        if setup_complete {
            if let Some(download_config) = game_info.find_download_config_by_name(name) {
                if download_config.setup {
                    continue;
                }
            }
        }

        if hash_check_install {
            if let Some(install_file_path) = find_cached_file(cache_dir, file) {
                let status_obj = client::StatusObj {
                    log_line: Some(format!("Checking install for {}", name)),
                    ..Default::default()
                };
                let status_str = serde_json::to_string(&status_obj).unwrap();
                sender.send(status_str).unwrap();

                let mut hash_file_path = std::format!("{}.hash", name);

                if let Some(file_download_config) = game_info.find_download_config_by_name(name) {
                    if let Some(tmp_extract_location) = file_download_config.extract_location {
                        let hashed_extract_location =
                            generate_hash_from_string(&tmp_extract_location)?;
                        hash_file_path = std::format!("{}-{}.hash", name, hashed_extract_location);
                        info!(
                            "hash_check_install extract location with config {}",
                            tmp_extract_location
                        );
                    }
                }

                info!(
                    "hash_check_install is enabled, checking for {}, game_command_file_found: {}",
                    name, game_command_file_found
                );

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
                        if game_command_file_found {
                            info!("hash for {} is same, skipping install", name);

                            let status_obj = client::StatusObj {
                                log_line: Some(format!(
                                    "Skipping install for {}, as hash is the same",
                                    name
                                )),
                                ..Default::default()
                            };
                            let status_str = serde_json::to_string(&status_obj).unwrap();
                            sender.send(status_str).unwrap();

                            continue;
                        } else {
                            info!(
                                "ignoring hash match for {}, since game command file was not found",
                                name
                            );
                        }
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
                match unpack_tarball(&path, game_info, name, sender) {
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

pub fn get_game_info(app_id: &str) -> io::Result<package_metadata::Game> {
    let package_metadata = package_metadata::PackageMetadata::from_packages_file();
    let game_info = package_metadata.find_game_by_app_id(app_id);

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
                    && (game_info.is_none()
                        || (!user_parsed["override_all_with_user_default"].is_null()
                            && user_parsed["override_all_with_user_default"] == true))
                {
                    info!("game info using user default");
                    match serde_json::from_str::<package_metadata::Game>(&json::stringify(
                        user_parsed["default"].clone(),
                    )) {
                        Ok(game) => return Ok(game),
                        Err(err) => {
                            let error_message =
                                std::format!("error parsing user parsed default: {:?}", err);
                            error!("{:?}", error_message);
                            return Err(Error::new(ErrorKind::Other, error_message));
                        }
                    }
                }
            } else {
                info!("user_packages_file used for game_info");
                match serde_json::from_str::<package_metadata::Game>(&json::stringify(
                    user_game_info,
                )) {
                    Ok(game) => return Ok(game),
                    Err(err) => {
                        let error_message =
                            std::format!("error parsing user parsed default: {:?}", err);
                        error!("{:?}", error_message);
                        return Err(Error::new(ErrorKind::Other, error_message));
                    }
                }
            }
        }
        None => {
            info!("user_packages_file not found");
        }
    };

    if let Some(ret_game_info) = game_info {
        Ok(ret_game_info)
    } else {
        info!("game info using default");
        Ok(package_metadata.default_engine)
    }
}

pub fn get_app_id_deps_paths(
    deps: &Vec<u32>,
    retry: bool,
    sender: &std::sync::mpsc::Sender<String>,
) -> io::Result<()> {
    let mut steam_app_id_install_completed = false;
    match SteamDir::locate() {
        Some(mut steamdir) => {
            for app_id in deps {
                info!("get_app_id_deps_paths. searching for app id {}.", app_id);

                match steamdir.app(app_id) {
                    Some(app_location) => {
                        let app_location_path = app_location.path.clone();

                        if app_location_path.exists()
                            && app_location_path.read_dir()?.next().is_some()
                        {
                            let app_location_str =
                                &app_location_path.into_os_string().into_string().unwrap();
                            let info_message = std::format!(
                                "get_app_id_deps_paths. app id {} found at {:#?}.",
                                app_id,
                                app_location_str
                            );
                            info!("{}", info_message);
                            user_env::set_env_var(
                                &std::format!("DEPPATH_{}", app_id).to_string(),
                                app_location_str,
                            );

                            let status_obj = client::StatusObj {
                                log_line: Some(info_message),
                                ..Default::default()
                            };
                            let status_str = serde_json::to_string(&status_obj).unwrap();
                            sender.send(status_str).unwrap();
                        } else {
                            match get_app_id_dep_path_retry(app_id, retry, &sender.clone()) {
                                Ok(()) => steam_app_id_install_completed = true,
                                Err(err) => {
                                    return Err(err);
                                }
                            }
                        }
                    }
                    None => match get_app_id_dep_path_retry(app_id, retry, &sender.clone()) {
                        Ok(()) => steam_app_id_install_completed = true,
                        Err(err) => {
                            return Err(err);
                        }
                    },
                }
            }

            if steam_app_id_install_completed {
                get_app_id_deps_paths(deps, true, sender)
            } else {
                Ok(())
            }
        }
        None => {
            let error_message = "get_app_id_deps_paths. steamdir not found.";
            error!("{}", error_message);
            Err(Error::new(ErrorKind::Other, error_message))
        }
    }
}

pub fn get_app_id_dep_path_retry(
    app_id: &u32,
    retry: bool,
    sender: &std::sync::mpsc::Sender<String>,
) -> io::Result<()> {
    let error_message = std::format!(
        "get_app_id_deps_paths. app id {} not found. retry = {}",
        app_id,
        retry
    );
    info!("{}", error_message);

    if retry {
        Err(Error::new(ErrorKind::Other, error_message))
    } else {
        let status_obj = client::StatusObj {
            log_line: Some(std::format!(
                "get_app_id_deps_paths. app id {} requesting install. This game needs this dependency to work, please press install on the steam dialog and wait for the install to complete. Luxtorpeda will launch the game when ready.",
                app_id
            )),
            ..Default::default()
        };
        let status_str = serde_json::to_string(&status_obj).unwrap();
        sender.send(status_str).unwrap();
        match request_steam_app_id_install(app_id) {
            Ok(()) => {
                let info_message = std::format!("steam_app_id_install_completed for {}", app_id);
                info!("{}", info_message);
                let status_obj = client::StatusObj {
                    log_line: Some(info_message),
                    ..Default::default()
                };
                let status_str = serde_json::to_string(&status_obj).unwrap();
                sender.send(status_str).unwrap();
                Ok(())
            }
            Err(err) => {
                let error_message = std::format!(
                    "get_app_id_deps_paths. app id {} not found. error = {:?}",
                    app_id,
                    err
                );
                error!("{}", error_message);
                Err(Error::new(ErrorKind::Other, error_message))
            }
        }
    }
}

pub fn request_steam_app_id_install(app_id: &u32) -> io::Result<()> {
    match Command::new("xdg-open")
        .args([std::format!("steam://install/{}", app_id)])
        .spawn()
    {
        Ok(mut child) => {
            match child.wait() {
                Ok(status) => {
                    info!("request_steam_app_id_install returned with {}", status);
                    let mut tries = 1;
                    let mut found_app = false;

                    let config = config::Config::from_config_file();
                    let num_tries = config.steam_app_id_install_wait_in_seconds / 5;

                    while tries < num_tries {
                        // wait for 5 seconds
                        info!(
                            "request_steam_app_id_install checking app of {} installed tries = {}",
                            app_id, tries
                        );
                        if let Some(mut steamdir) = SteamDir::locate() {
                            if let Some(app_metadata) = steamdir.app(app_id) {
                                if let Some(state_flags) = &app_metadata.state_flags {
                                    for state_flag in state_flags.iter() {
                                        if let steamlocate::steamapp::StateFlag::FullyInstalled =
                                            state_flag
                                        {
                                            info!(
                                                "request_steam_app_id_install found app location of {}",
                                                app_id
                                                );
                                            found_app = true;
                                            break;
                                        }
                                    }

                                    if found_app {
                                        break;
                                    }
                                }
                            }
                        }
                        tries += 1;
                        thread::sleep(Duration::from_secs(5));
                    }

                    if found_app {
                        Ok(())
                    } else {
                        let error_message = std::format!(
                            "request_steam_app_id_install. app not found of {}.",
                            app_id
                        );
                        error!("{}", error_message);
                        Err(Error::new(ErrorKind::Other, error_message))
                    }
                }
                Err(err) => Err(err),
            }
        }
        Err(err) => Err(err),
    }
}

pub fn install_steam_input_template(app_id: &u32, steam_input_template_path: &Path) {
    match SteamDir::locate() {
        Some(mut steamdir) => {
            info!(
                "install_steam_input_template. searching for app id {}.",
                app_id
            );

            match steamdir.app(app_id) {
                Some(app_location) => {
                    info!("install_steam_input_template. app id {} found", app_id);
                    if let Some(last_user) = app_location.last_user {
                        let account_id = last_user.account_id();
                        info!(
                            "install_steam_input_template. app id {} last_user found {:?}, account_id = {}", app_id, last_user, account_id);

                        if let Some(steam_path) = user_env::steam_install_path() {
                            let input_folder_path = Path::new(&steam_path)
                                .join("userdata")
                                .join(account_id.to_string())
                                .join("241100")
                                .join("remote")
                                .join("controller_config")
                                .join(app_id.to_string());

                            let full_input_path =
                                input_folder_path.join("steam_input_template.vdf");

                            if !full_input_path.exists() {
                                info!(
                                    "install_steam_input_template. full_input_path = {}",
                                    full_input_path.display()
                                );

                                match fs::create_dir_all(input_folder_path.clone()) {
                                    Ok(()) => {
                                        match fs::copy(steam_input_template_path, full_input_path) {
                                            Ok(_) => {
                                                info!(
                                                    "install_steam_input_template copy successful"
                                                );
                                            }
                                            Err(err) => {
                                                warn!(
                                                    "install_steam_input_template. copy error {:?}",
                                                    err
                                                );
                                            }
                                        }
                                    }
                                    Err(err) => {
                                        warn!("install_steam_input_template. create_dir_all error {:?}", err);
                                    }
                                }
                            } else {
                                info!(
                            "install_steam_input_template. full_input_path {} already exists", full_input_path.display());
                            }
                        }
                    }
                }
                None => {
                    info!("install_steam_input_template. app id {} not found.", app_id);
                }
            }
        }
        None => {
            info!("install_steam_input_template. steamdir not found.");
        }
    }
}
