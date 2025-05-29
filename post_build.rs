use std::env;
use std::path::Path;
extern crate fs_extra;
use fs_extra::copy_items;
use std::fs;
use std::process::Command;
extern crate tar;
use xz2::write::XzEncoder;

// These variables are used to generate compatibilitytool.vdf
const TOOL_NAME: &str = "luxtorpeda";
const TOOL_NAME_DEV: &str = "luxtorpeda_dev";
const TOOL_NAME_DISPLAY: &str = "Luxtorpeda";
const TOOL_NAME_DISPLAY_DEV: &str = "Luxtorpeda (dev)";

// Files that should be copied from the root of the repo
const ROOT_FILES: &[&str] = &[
    "toolmanifest.vdf",
    "LICENSE",
    "README.md",
    "luxtorpeda.sh"
];

// Files that should be copied from target to the final destination
const FILES: &[&str] = &[
    "compatibilitytool.vdf",
    "libluxtorpeda.so",
    "godot_export/luxtorpeda.pck",
    "godot_export/luxtorpeda.x86_64",
];

fn main() {
    let out_dir = env::var("CRATE_OUT_DIR").unwrap();
    let profile = env::var("CRATE_PROFILE").unwrap();

    let godot_export_path = Path::new(&out_dir).join("godot_export");
    fs::create_dir_all(&godot_export_path).expect("Failed to create godot_export dir");
    let godot_export_str = godot_export_path
        .to_str()
        .expect("Invalid Unicode in path")
        .to_owned();

    create_target_gdignore(&out_dir);
    if profile == "release" {
        release_godot_workaround(&out_dir);
    }

    match env::var("GODOT") {
        Ok(godot_path) => build_godot_project(&godot_export_str, &godot_path, &profile),
        Err(_) => {
            eprintln!("godot not provided so skipping");
        }
    };

    create_compatibilitytool_vdf(&out_dir, &profile);
    copy_root_files(&out_dir);

    match env::var("TARGET") {
        Ok(target) => {
            if !target.is_empty() {
                resolve_target(&out_dir, &target);
            }
        },
        Err(_) => {
            eprintln!("target not provided so skipping");
        }
    }
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

fn build_godot_project(out_dir: &str, godot_path: &str, profile: &str) {
    println!("build_godot_project");
    let out_path = Path::new(out_dir).join("luxtorpeda.x86_64").into_os_string().into_string().unwrap();
    let build_cmd = Command::new(godot_path)
        .args(["--path", ".", &std::format!("--export-{}", profile).to_string(), "Linux/X11", &out_path, "--display-driver", "headless"])
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

    let tool_name = match profile {
        "debug" => TOOL_NAME_DEV,
        _ => TOOL_NAME
    };

    let mut file_str = template_str.replace("%display_name%", display_name);
    file_str = file_str.replace("%name%", tool_name);
    fs::write(output_path, file_str).expect("create_compatibilitytool_vdf write error");
}

fn resolve_target(out_dir: &str, target: &str) {
    println!("resolve_target, target is {}", target);

    build_folder(&out_dir, TOOL_NAME);

    match target {
        "luxtorpeda" => {},
        "luxtorpeda.tar.xz" => {
            create_archive(TOOL_NAME, &target);
        },
        "user_install" => {},
        "install" => {},
        _ => {
            eprintln!("resolve_target - Unknown target of {}", target);
        }
    };
}

fn build_folder(out_dir: &str, folder_path: &str) {
    println!("build_folder with {}", folder_path);
    fs::create_dir_all(folder_path).expect("build_tool_folder create dir failed");

    let options = fs_extra::dir::CopyOptions {
        overwrite: true,
        copy_inside: true,
        ..Default::default()
    };

    let mut files = Vec::new();
    for filename in ROOT_FILES.iter() {
        files.push(Path::new(out_dir).join(filename));
    }
    for filename in FILES.iter() {
        files.push(Path::new(out_dir).join(filename));
    }

    copy_items(&files, folder_path, &options).expect("build_tool_folder copy failed");

    match env::var("VERSION") {
        Ok(version) => {
            if !version.is_empty() {
                fs::write(Path::new(folder_path).join("version"), version).expect("build_folder version write error");
            }
        },
        Err(_) => {
            eprintln!("version not provided so skipping");
        }
    }
}

fn create_archive(folder_path: &str, archive_name: &str) {
    println!("create_archive from folder {}", folder_path);

    let tar_xz = std::fs::File::create(archive_name).expect("create_archive file create error");
    let enc = XzEncoder::new(tar_xz, 6);
    let mut tar = tar::Builder::new(enc);
    tar.append_dir_all(TOOL_NAME, folder_path).expect("create_archive append error");
}
