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
    find_cached_file(app_id, "dist.tar.xz").is_some()
}

pub fn download(app_id: &str) -> io::Result<()> {
    println!("downloading package for app_id {:}", app_id);

    let target = "https://luxtorpeda.gitlab.io/packages/ioq3/dist.tar.xz";
    match reqwest::get(target) {
        Ok(mut response) => {
            let dest_file = place_cached_file(app_id, "test.tar.xz")?;
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
