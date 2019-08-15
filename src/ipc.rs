use std::fs;
use std::io::prelude::*;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use std::time;

pub enum StatusMsg {
    Status(i32, i32, String),
    Done,
}

fn socket_path(app_id: &str) -> PathBuf {
    let xdg_dirs = xdg::BaseDirectories::new().unwrap();
    let path_str = format!("luxtorpeda/{}.socket", app_id);
    let path = xdg_dirs.place_runtime_file(&path_str);
    assert!(xdg_dirs.has_runtime_directory());
    path.unwrap()
}

pub fn query_status(app_id: String) {
    let socket_path = socket_path(app_id.as_str());

    let mut stream = match UnixStream::connect(socket_path) {
        Ok(stream) => stream,
        Err(_) => return,
    };

    let mut status = String::new();
    stream.read_to_string(&mut status).unwrap();
    print!("{}", status); // "0/1: <luxtorpeda game package>"
}

pub fn status_relay(rx: Receiver<StatusMsg>, app_id: String) {
    let socket_path = socket_path(app_id.as_str());

    // delete old socket if necessary
    if socket_path.exists() {
        fs::remove_file(&socket_path).unwrap(); // unlink
    }

    let listener = match UnixListener::bind(&socket_path) {
        Err(_) => panic!("failed to bind socket"),
        Ok(l) => l,
    };

    listener
        .set_nonblocking(true)
        .expect("Couldn't set non blocking");

    let mut msg = String::from("");

    loop {
        match rx.recv_timeout(time::Duration::from_millis(20)) {
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Ok(StatusMsg::Status(i, n, name)) => {
                msg = format!("{}/{}: {}", i, n, name);
                println!("got {}", msg);
            }
            Ok(StatusMsg::Done) => break,
            _ => break,
        }

        match listener.accept() {
            Ok((mut stream, _)) => match stream.write_all(msg.as_bytes()) {
                Err(e) => println!("send err: {}", e),
                Ok(()) => {}
            },
            _ => {}
        }
    }

    fs::remove_file(socket_path).unwrap(); // unlink
}
