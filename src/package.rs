extern crate reqwest;
extern crate tar;
extern crate xz2;

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use tar::Archive;
use xz2::read::XzDecoder;
use dialog::DialogBox;

use crate::ipc;
use crate::user_env;

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
}

pub fn read_cmd_repl_from_file<P: AsRef<Path>>(path: P) -> Result<Vec<CmdReplacement>, Error> {
    let file = fs::File::open(path)?;
    let reader = io::BufReader::new(file);
    let meta: PackageMetadata = serde_json::from_reader(reader)?;
    Ok(meta.commands)
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

        if find_cached_file(app_id, file.as_str()).is_some() {
            println!("{} found in cache (skip)", file);
            continue;
        }

        downloads.push(PackageInfo { name, url, file });
    }
    Ok(downloads)
}

pub fn download_all(app_id: String) -> io::Result<()> {
    let game_info = get_game_info(app_id.as_str())
        .ok_or_else(|| Error::new(ErrorKind::Other, "missing info about this game"))?;

    if game_info["download"].is_null() {
        println!("skipping downloads (no urls defined for this package)");
        return Ok(());
    }

    let downloads = json_to_downloads(app_id.as_str(), &game_info)?;

    if downloads.is_empty() {
        return Ok(());
    }
    
    if !game_info["information"].is_null() && game_info["information"]["non_free"] == true {
        let dialog_message = std::format!("This engine uses a non-free engine ({0}). Are you sure you want to continue?", game_info["information"]["license"]);
        let choice = dialog::Question::new(dialog_message)
            .title("Non-Free License Warning")
            .show()
            .expect("Could not display dialog box");
        
        if choice == dialog::Choice::No || choice == dialog::Choice::Cancel {
            println!("show_non_free_dialog. dialog was rejected");
            return Err(Error::new(ErrorKind::Other, "dialog was rejected"));
        }
    }
    
    if !game_info["information"].is_null() && game_info["information"]["closed_source"] == true {
        let dialog_message = "This engine uses assets from the closed source release. Are you sure you want to continue?";
        let choice = dialog::Question::new(dialog_message)
            .title("Closed Source Engine Warning")
            .show()
            .expect("Could not display dialog box");
        
        if choice == dialog::Choice::No || choice == dialog::Choice::Cancel {
            println!("show_non_free_dialog. dialog was rejected");
            return Err(Error::new(ErrorKind::Other, "dialog was rejected"));
        }
    }

    let (tx, rx): (Sender<ipc::StatusMsg>, Receiver<ipc::StatusMsg>) = mpsc::channel();

    let app = app_id.clone();
    let status_relay = thread::spawn(move || {
        ipc::status_relay(rx, app);
    });

    let mut err = Ok(());
    let n = downloads.len() as i32;
    for (i, info) in downloads.iter().enumerate() {
        // update status
        //
        match tx.send(ipc::StatusMsg::Status(i as i32, n, info.name.clone())) {
            Ok(()) => {}
            Err(e) => {
                print!("err: {}", e);
            }
        }
        err = download(app_id.as_str(), info);
    }

    // stop relay thread
    //
    match tx.send(ipc::StatusMsg::Done) {
        Ok(()) => {}
        Err(e) => {
            print!("err: {}", e);
        }
    };

    status_relay.join().expect("status relay thread panicked");

    err
}

fn download(app_id: &str, info: &PackageInfo) -> io::Result<()> {
    let target = info.url.clone() + &info.file;

    // TODO handle 404 and other errors
    //
    match reqwest::get(target.as_str()) {
        Ok(mut response) => {
            let dest_file = place_cached_file(app_id, &info.file)?;
            let mut dest = fs::File::create(dest_file)?;
            io::copy(&mut response, &mut dest)?;
            Ok(())
        }
        Err(err) => {
            println!("download err: {:?}", err);
            Err(Error::new(ErrorKind::Other, "download error"))
        }
    }
}

fn unpack_tarball(tarball: PathBuf) -> io::Result<()> {
    let package_name = tarball
        .file_name()
        .and_then(|x| x.to_str())
        .and_then(|x| x.split('.').next())
        .ok_or_else(|| Error::new(ErrorKind::Other, "package has no name?"))?;

    let transform = |path: PathBuf| -> PathBuf {
        match path.as_path().to_str() {
            Some("manifest.json") => PathBuf::from(format!("manifests.lux/{}.json", &package_name)),
            _ => PathBuf::from(path.strip_prefix("dist").unwrap_or(&path)),
        }
    };

    eprintln!("installing: {}", package_name);

    let file = fs::File::open(&tarball)?;
    let mut archive = Archive::new(XzDecoder::new(file));

    for entry in archive.entries()? {
        let mut file = entry?;
        let old_path = PathBuf::from(file.header().path()?);
        let new_path = transform(old_path);
        if new_path.to_str().map_or(false, |x| x.is_empty()) {
            continue;
        }
        println!("install: {:?}", &new_path);
        if new_path.parent().is_some() {
            fs::create_dir_all(new_path.parent().unwrap())?;
        }
        let _ = fs::remove_file(&new_path);
        file.unpack(&new_path)?;
    }

    Ok(())
}

pub fn install() -> io::Result<()> {
    let app_id = user_env::steam_app_id();

    let game_info = get_game_info(app_id.as_str())
        .ok_or_else(|| Error::new(ErrorKind::Other, "missing info about this game"))?;

    let packages: Vec<String> = game_info["download"]
        .members()
        .map(|j| j["file"].to_string())
        .collect();

    for file in packages {
        match find_cached_file(&app_id, &file) {
            Some(path) => {
                unpack_tarball(path)?;
            }
            None => {
                return Err(Error::new(ErrorKind::Other, "package file not found"));
            }
        }
    }
    Ok(())
}

pub fn get_game_info(app_id: &str) -> Option<json::JsonValue> {
    let packages_json_file = user_env::tool_dir().join("packages.json");
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
    if game_info.is_null() {
        None
    } else {
        Some(game_info)
    }
}
