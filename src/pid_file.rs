extern crate xdg;

use std::path::PathBuf;

use crate::user_env;


pub struct PidFile {
    name: PathBuf,
}

fn pid_file_path() -> PathBuf {
    let xdg_dirs = xdg::BaseDirectories::new().unwrap();
    let path_str = format!("luxtorpeda/{}.pid", user_env::steam_app_id());
    let path = xdg_dirs.place_runtime_file(&path_str);
    assert!(xdg_dirs.has_runtime_directory());
    path.unwrap()
}

pub fn new() -> PidFile {
    let name = pid_file_path();
    println!("creating: {:?}", name);
    PidFile { name }
    // TODO use psutil::write_pidfile here
}

impl Drop for PidFile {

    fn drop(&mut self) {
        println!("dropping: {:?}", self.name);
    }

}
