extern crate reqwest;

use std::fs;
use std::io;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;
use std::process::Command;

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

pub fn is_cached(app_id: &str) -> bool {
    match get_game_info(app_id) {
        Some(game_info) => {
            if game_info["package"].is_null() {
                false
            } else {
                let package = game_info["package"].to_string();
                find_cached_file(app_id, &package).is_some()
            }
        }
        None => false,
    }
}

pub fn download(app_id: &str) -> io::Result<()> {
    let game_info = get_game_info(app_id)
        .ok_or(Error::new(ErrorKind::Other, "missing info about this game"))?;

    if game_info["package_url"].is_null() {
        println!("skipping download (no url defined for this package)");
        return Ok(());
    }

    if game_info["package"].is_null() {
        println!("url defined, but package name missing");
        return Err(Error::new(ErrorKind::Other, "missing package name"));
    }

    println!("downloading package for app_id {:}", app_id);

    let url = game_info["package_url"].to_string();
    let package = game_info["package"].to_string();
    let target = url + &package;

    // TODO handle 404 and other errors
    //
    match reqwest::get(target.as_str()) {
        Ok(mut response) => {
            let dest_file = place_cached_file(app_id, &package)?;
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
