extern crate inotify;
extern crate psutil;
extern crate xdg;

use std::fs;
use std::io;
use std::path::PathBuf;

use inotify::{Inotify, WatchMask};

use psutil::pidfile;
use psutil::process::Process;

use crate::user_env;

pub struct PidFile {
    path: PathBuf,
}

fn pid_file_path() -> PathBuf {
    let xdg_dirs = xdg::BaseDirectories::new().unwrap();
    let path_str = format!("luxtorpeda/{}.pid", user_env::steam_app_id());
    let path = xdg_dirs.place_runtime_file(&path_str);
    assert!(xdg_dirs.has_runtime_directory());
    path.unwrap()
}

pub fn new() -> io::Result<PidFile> {
    let path = pid_file_path();
    println!("creating: {:?}", path);

    let is_other_process_alive = match Process::from_pidfile(&path) {
        Ok(p) => p.is_alive(),
        Err(_) => false,
    };

    if is_other_process_alive {
        let err = io::Error::new(io::ErrorKind::Other, "Other process is alive");
        return Err(err);
    }

    pidfile::write_pidfile(&path)?;
    Ok(PidFile { path })
}

impl Drop for PidFile {
    fn drop(&mut self) {
        println!("dropping: {:?}", &self.path);
        match fs::remove_file(&self.path) {
            Err(e) => println!("err: {:?}", e),
            Ok(()) => {}
        }
    }
}

/// Wait until pid file is deleted by previous Luxtorpeda invocation.
///
pub fn wait_while_exists() {
    let path = pid_file_path();
    let pid = match pidfile::read_pidfile(&path) {
        Ok(p) => p,
        Err(_) => return,
    };
    let is_alive = match Process::new(pid) {
        Ok(p) => p.is_alive(),
        Err(_) => false,
    };

    if !is_alive {
        return;
    }

    let mut inotify = Inotify::init().expect("Failed to initialize inotify");

    inotify
        .add_watch(&path, WatchMask::DELETE)
        .expect("failed to add inotify watch");

    println!("waiting for process {:?} to stop \
              and delete file {:?}", pid, path);

    let mut buffer = [0; 128];
    inotify
        .read_events_blocking(&mut buffer)
        .expect("error while reading events");
}
