use std::env;
use std::path::Path;
extern crate fs_extra;
use fs_extra::copy_items;
use std::fs;
use std::process::Command;

// TODO: destdir, use from make file if provided, skip install if not, for use in the install commands

// These variables are used to generate compatibilitytool.vdf
const TOOL_NAME: &str = "luxtorpeda";
const TOOL_NAME_DEV: &str = "luxtorpeda_dev";
const TOOL_NAME_DISPLAY: &str = "Luxtorpeda";
const TOOL_NAME_DISPLAY_DEV: &str = "Luxtorpeda (dev)";

// Files that should be copied from the root of the repo
const ROOT_FILES: &[&str] = &[
    "toolmanifest.vdf",
    "LICENSE",
    "README.md"
];

// Files that should be copied from target to the final destination
const FILES: &[&str] = &[
    "compatibilitytool.vdf",
    "libluxtorpeda.so",
    "luxtorpeda.pck",
    "luxtorpeda.sh",
    "luxtorpeda.x86_64",
];

fn main() {
    let out_dir = env::var("CRATE_OUT_DIR").unwrap();
    let profile = env::var("CRATE_PROFILE").unwrap();

    for (key, value) in env::vars() {
        println!("{key}: {value}");
    }

    create_target_gdignore(&out_dir);
    if profile == "release" {
        release_godot_workaround(&out_dir);
    }
    copy_root_files(&out_dir);

    match env::var("GODOT") {
        Ok(godot_path) => build_godot_project(&out_dir, &godot_path),
        Err(err) => {
            eprintln!("godot not provided so skipping");
        }
    };

    create_compatibilitytool_vdf(&out_dir, &profile);
}

fn create_target_gdignore(out_dir: &str) {
    std::fs::File::create(Path::new(out_dir).join(".gdignore")).expect("create gdignore failed");
}

fn release_godot_workaround(out_dir: &str) {
    println!("release_godot_workaround, copying files");
    let options = fs_extra::dir::CopyOptions {
        overwrite: true,
        ..Default::default()
    };
    let debug_dir = Path::new(out_dir).parent().unwrap().join("debug");
    create_target_gdignore(out_dir);
    fs::create_dir_all(debug_dir.clone()).expect("failed to create debug dir");

    let mut from_paths = Vec::new();
    from_paths.push(Path::new(out_dir).join("libluxtorpeda.so"));
    copy_items(&from_paths, debug_dir, &options).expect("release_godot_workaround copy failed");
}

fn copy_root_files(out_dir: &str) {
    println!("copy_root_files");

    let options = fs_extra::dir::CopyOptions {
        overwrite: true,
        ..Default::default()
    };
    copy_items(&ROOT_FILES, out_dir, &options).expect("copy_root_files copy failed");
}

fn build_godot_project(out_dir: &str, godot_path: &str) {
    println!("build_godot_project");
    let out_path = Path::new(out_dir).join("luxtorpeda.x86_64").into_os_string().into_string().unwrap();
    let build_cmd = Command::new(godot_path)
        .args(["--path", ".", "--export", "Linux/X11", &out_path, "--no-window"])
        .status()
        .expect("failed to execute godot");

    if !build_cmd.success() {
        panic!("build_godot_project failed");
    }
}

fn create_compatibilitytool_vdf(out_dir: &str, profile: &str) {
    println!("create_compatibilitytool_vdf");
    let template_str = fs::read_to_string("compatibilitytool.template").expect("create_compatibilitytool_vdf read template error");

    let output_path = Path::new(out_dir).join("compatibilitytool.vdf");
    let display_name = match profile {
        "debug" => TOOL_NAME_DISPLAY_DEV,
        _ => TOOL_NAME_DISPLAY
    };
    let file_str = template_str.replace("%display_name%", display_name);
    fs::write(output_path, file_str).expect("create_compatibilitytool_vdf write error");
}
