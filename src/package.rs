extern crate reqwest;

use std::fs;
use std::io;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;
use std::process::Command;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

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

struct PackageInfo {
    name: String,
    url: String,
    file: String,
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

        downloads.push(PackageInfo {
            name: name,
            url: url,
            file: file,
        });
    }
    Ok(downloads)
}

pub fn download_all(app_id: String) -> io::Result<()> {
    let game_info = get_game_info(app_id.as_str())
        .ok_or(Error::new(ErrorKind::Other, "missing info about this game"))?;

    if game_info["download"].is_null() {
        println!("skipping downloads (no urls defined for this package)");
        return Ok(());
    }

    let downloads = json_to_downloads(app_id.as_str(), &game_info)?;

    if downloads.is_empty() {
        return Ok(());
    }

    let (tx, rx): (Sender<ipc::StatusMsg>, Receiver<ipc::StatusMsg>) = mpsc::channel();

    let app = app_id.clone();
    let status_relay = thread::spawn(move || {
        ipc::status_relay(rx, app);
    });

    let mut err = Ok(());
    for info in downloads {
        // update status
        //
        match tx.send(ipc::StatusMsg::Status(0, 1, info.name.clone())) {
            // TODO: 0/1 ??????
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

fn download(app_id: &str, info: PackageInfo) -> io::Result<()> {
    let target = info.url + &info.file;

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

pub fn install(package: String) -> io::Result<()> {
    let app_id = user_env::steam_app_id();
    match find_cached_file(&app_id, &package) {
        Some(path) => {
            Command::new("tar")
                .arg("xJf")
                .arg(path)
                .arg("--strip-components=1")
                .arg("dist")
                .status()
                .expect("package installation failed");
            Ok(())
        }
        None => Err(Error::new(ErrorKind::Other, "package file not found")),
    }
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
