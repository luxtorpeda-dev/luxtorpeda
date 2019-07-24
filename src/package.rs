use std::io;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;
use std::process::Command;

use crate::user_env;

fn find_cached_file(file: String) -> Option<PathBuf> {
    let xdg_dirs = xdg::BaseDirectories::new().unwrap();
    let path_str = format!("luxtorpeda/{}/{}", user_env::steam_app_id(), file);
    xdg_dirs.find_cache_file(path_str)
}

pub fn install(package: String) -> io::Result<()> {
    match find_cached_file(package) {
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
