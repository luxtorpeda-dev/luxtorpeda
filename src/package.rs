extern crate reqwest;

use std::io;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;
use std::process::Command;

use crate::user_env;

fn find_cached_file(app_id: &String, file: &String) -> Option<PathBuf> {
    let xdg_dirs = xdg::BaseDirectories::new().unwrap();
    let path_str = format!("luxtorpeda/{}/{}", app_id, file);
    xdg_dirs.find_cache_file(path_str)
}

pub fn is_cached(app_id: &String) -> bool {
    find_cached_file(app_id, &"dist.tar.xz".to_string()).is_some()
}

pub fn download(app_id: &String) -> io::Result<()> {
    println!("downloading {:}", app_id);
    Ok(())
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
