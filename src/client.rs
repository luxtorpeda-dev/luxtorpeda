extern crate reqwest;

use gdnative::prelude::*;
use std::sync::mpsc::channel;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::env;
use std::io;
use std::fs;
use std::io::{Error, ErrorKind};
use tokio::runtime::Runtime;
use std::cmp::min;
use std::io::Write;
use reqwest::Client;
use futures_util::StreamExt;

use crate::user_env;
use crate::package;
use crate::package::ChoiceInfo;

#[derive(NativeClass)]
#[inherit(Node)]
// register_with attribute can be used to specify custom register function for node signals and properties
#[register_with(Self::register_signals)]
pub struct SignalEmitter;

#[methods]
impl SignalEmitter {
    fn register_signals(builder: &ClassBuilder<Self>) {
        builder.signal("choice_picked").done();
    }

    fn new(_owner: &Node) -> Self {
        SignalEmitter
    }
}

#[derive(NativeClass)]
#[inherit(Node)]
pub struct LuxClient
{
    receiver: std::option::Option<std::sync::mpsc::Receiver<String>>
}

#[derive(Serialize, Deserialize, Debug)]
struct StatusObj {
    label: std::option::Option<String>,
    progress: std::option::Option<i64>
}

#[methods]
impl LuxClient {
    fn new(_base: &Node) -> Self {
        LuxClient { receiver: None }
    }

    #[method]
    fn _ready(&self, #[base] base: TRef<Node>) {
        let app_id = user_env::steam_app_id();
        let env_args: Vec<String> = env::args().collect();
        let args: Vec<&str> = env_args.iter().map(|a| a.as_str()).collect();
        let cmd_args = &args[2..];

        info!("luxtorpeda version: {}", env!("CARGO_PKG_VERSION"));
        info!("steam_app_id: {:?}", &app_id);
        info!("original command: {:?}", args);
        info!("working dir: {:?}", env::current_dir());
        info!("tool dir: {:?}", user_env::tool_dir());

        if args.len() < 2 {
            //usage();
            std::process::exit(0)
        }

        let exe = cmd_args[0].to_lowercase();
        let exe_args = &cmd_args[1..];

        if exe.ends_with("iscriptevaluator.exe") {
            std::process::exit(0)
        }

        user_env::assure_xdg_runtime_dir();
        user_env::assure_tool_dir(args[0]);

        let emitter = &mut base.get_node("SignalEmitter").unwrap();
        let emitter = unsafe { emitter.assume_safe() };

        emitter
            .connect(
                "choice_picked",
                base,
                "choice_picked",
                VariantArray::new_shared(),
                0,
            )
            .unwrap();

        package::update_packages_json();
        self.ask_for_engine_choice(app_id.as_str(), &base);
    }

    #[method]
    fn _physics_process(&self, #[base] base: TRef<Node>, _delta: f64) {
        if let Some(receiver) = &self.receiver {
            if let Ok(new_data) = receiver.try_recv() {
                let emitter = &mut base.get_node("Container/Progress").unwrap();
                let emitter = unsafe { emitter.assume_safe() };
                emitter.emit_signal("progress_change", &[Variant::new(new_data)]);
            }
        }
    }

    fn ask_for_engine_choice(&self, app_id: &str, owner: &Node) -> io::Result<String> {
        let mut game_info = package::get_game_info(app_id)
            .ok_or_else(|| Error::new(ErrorKind::Other, "missing info about this game"))?;


        if game_info.is_null() {
            return Err(Error::new(ErrorKind::Other, "Unknown app_id"));
        }

        if !game_info["choices"].is_null() {
            let engines_option = package::get_engines_info();

            let mut choices: Vec<ChoiceInfo> = vec![];
            for entry in game_info["choices"].members() {
                if entry["name"].is_null() {
                    return Err(Error::new(ErrorKind::Other, "missing choice info"));
                }

                let mut choice_info = ChoiceInfo {
                    name: entry["name"].to_string(),
                    notices: Vec::new(),
                };

                let mut engine_name = entry["name"].to_string();
                if !entry["engine_name"].is_null() {
                    engine_name = entry["engine_name"].to_string();
                }

                let engine_name_clone = engine_name.clone();
                if let Some((ref engines, ref notice_map)) = engines_option {
                    if !engines[engine_name_clone].is_null() {
                        let engine_name_clone_clone = engine_name.clone();
                        let engine_name_clone_clone_two = engine_name.clone();
                        let engine_name_clone_clone_three = engine_name.clone();
                        let engine_name_clone_clone_four = engine_name.clone();

                        if !engines[engine_name_clone_clone]["notices"].is_null() {
                            for entry in engines[engine_name]["notices"].members() {
                                choice_info
                                    .notices
                                    .push(package::convert_notice_to_str(entry, notice_map));
                            }
                        }

                        let controller_not_supported =
                            engines[engine_name_clone_clone_two]["controllerNotSupported"] == true;
                        let controller_supported =
                            engines[engine_name_clone_clone_three]["controllerSupported"] == true;
                        let controller_supported_manual =
                            engines[engine_name_clone_clone_four]["controllerSupportedManualGame"] == true;

                        if controller_not_supported {
                            choice_info
                                .notices
                                .push("Engine Does Not Have Native Controller Support".to_string());
                        } else if controller_supported && game_info["controllerSteamDefault"] == true {
                            choice_info.notices.push(
                                "Engine Has Native Controller Support And Works Out of the Box".to_string(),
                            );
                        } else if controller_supported_manual && game_info["controllerSteamDefault"] == true
                        {
                            choice_info.notices.push(
                                "Engine Has Native Controller Support But Needs Manual In-Game Settings"
                                    .to_string(),
                            );
                        } else if controller_supported
                            && (game_info["controllerSteamDefault"].is_null()
                                || game_info["controllerSteamDefault"] != true)
                        {
                            choice_info.notices.push(
                                "Engine Has Native Controller Support But Needs Manual Steam Settings"
                                    .to_string(),
                            );
                        }
                    }

                    if game_info["cloudNotAvailable"] == true {
                        choice_info
                            .notices
                            .push("Game Does Not Have Cloud Saves".to_string());
                    } else if game_info["cloudAvailable"] == true
                        && (game_info["cloudSupported"].is_null() || game_info["cloudSupported"] != true)
                    {
                        choice_info
                            .notices
                            .push("Game Has Cloud Saves But Unknown Status".to_string());
                    } else if game_info["cloudAvailable"] == true && game_info["cloudSupported"] == true {
                        choice_info
                            .notices
                            .push("Cloud Saves Supported".to_string());
                    } else if game_info["cloudAvailable"] == true && game_info["cloudIssue"] == true {
                        choice_info
                            .notices
                            .push("Cloud Saves Not Supported".to_string());
                    }

                    if !game_info["notices"].is_null() {
                        for entry in game_info["notices"].members() {
                            choice_info
                                .notices
                                .push(package::convert_notice_to_str(entry, notice_map));
                        }
                    }
                }

                choices.push(choice_info);
            }

            let choices_str = serde_json::to_string(&choices).unwrap();
            let emitter = &mut owner.get_node("Container/Choices").unwrap();
            let emitter = unsafe { emitter.assume_safe() };
            emitter.emit_signal("choices_found", &[Variant::new(choices_str)]);
        }

        Ok("test".to_string())
    }

    #[method]
    fn choice_picked(&mut self, #[base] owner: &Node, data: Variant) {

        info!("ASD*&&*&*&*&*");
        let msg = format!(
            "Received signal \"choice_picked\" with data {}",
            data.try_to::<String>().unwrap()
        );

        info!("ASDASDASD {}", msg);

        let app_id = user_env::steam_app_id();
        let mut game_info = package::get_game_info(app_id.as_str()).unwrap();

        let engine_choice = data.try_to::<String>().unwrap();

        /*if !game_info["app_ids_deps"].is_null() {
            match get_app_id_deps_paths(&game_info["app_ids_deps"]) {
                Some(()) => {
                    info!("download_all. get_app_id_deps_paths completed");
                }
                None => {
                    info!("download_all. warning: get_app_id_deps_paths not completed");
                }
            }
        }*/

        if game_info["download"].is_null() {
            info!("skipping downloads (no urls defined for this package)");
            return;
        }

        let downloads = package::json_to_downloads(app_id.as_str(), &game_info).unwrap();

        if downloads.is_empty() {
            return;
        }

        /*let mut dialog_message = String::new();

        if !game_info["information"].is_null() && game_info["information"]["non_free"] == true {
            dialog_message = std::format!(
                "This engine uses a non-free engine ({0}). Are you sure you want to continue?",
                game_info["information"]["license"]
            );
        } else if !game_info["information"].is_null()
            && game_info["information"]["closed_source"] == true
        {
            dialog_message = "This engine uses assets from the closed source release. Are you sure you want to continue?".to_string();
        }

        if !dialog_message.is_empty() {
            match show_question("License Warning", &dialog_message) {
                Some(_) => {
                    info!("show license warning. dialog was accepted");
                }
                None => {
                    info!("show license warning. dialog was rejected");
                    return Err(Error::new(ErrorKind::Other, "dialog was rejected"));
                }
            };
        }*/

        let (sender, receiver) = channel();

        std::thread::spawn(move || {
            let client = reqwest::Client::new();

            for (i, info) in downloads.iter().enumerate() {
                let app_id = app_id.to_string();
                info!("starting download on: {} {}", i, info.name.clone());

                let label_str = std::format!(
                    "Downloading {}/{} - {}",
                    i + 1,
                    downloads.len(),
                    info.name.clone()
                );

                let status_obj = StatusObj { label: Some(label_str), progress: None };
                let status_str = serde_json::to_string(&status_obj).unwrap();
                sender.send(status_str).unwrap();

                match Runtime::new().unwrap().block_on(Self::download(
                    app_id.as_str(),
                    info,
                    sender.clone(),
                    &client,
                )) {
                    Ok(_) => {}
                    Err(ref err) => {
                        error!("download of {} error: {}", info.name.clone(), err);
                       /* let mut guard = download_err_arc.lock().unwrap();
                        guard.close = true;
                        guard.error = true;

                        if err.to_string() != "progress update failed" {
                            guard.error_str =
                                std::format!("Download of {} Error: {}", info.name.clone(), err);
                        }

                        std::mem::drop(guard);*/

                        let mut cache_dir = app_id;
                        if info.cache_by_name {
                            cache_dir = info.name.clone();
                        }
                        let dest_file = package::place_cached_file(&cache_dir, &info.file).unwrap();
                        if dest_file.exists() {
                            fs::remove_file(dest_file).unwrap();
                        }
                    }
                };

                /*let error_check_arc = loop_arc.clone();
                let guard = error_check_arc.lock().unwrap();
                if !guard.error {
                    info!("completed download on: {} {}", i, info.name.clone());
                } else {
                    error!("failed download on: {} {}", i, info.name.clone());
                    std::mem::drop(guard);
                    break;
                }
                std::mem::drop(guard);
                }*/
            }
        });

        self.receiver = Some(receiver);
    }

    async fn download(
        app_id: &str,
        info: &package::PackageInfo,
        sender: std::sync::mpsc::Sender<String>,
        client: &Client,
    ) -> io::Result<()> {
        let target = info.url.clone() + &info.file;

        let mut cache_dir = app_id;
        if info.cache_by_name {
            cache_dir = &info.name;
        }

        info!("download target: {:?}", target);

        let res = client.get(&target).send().await.or(Err(Error::new(
            ErrorKind::Other,
            format!("Failed to GET from '{}'", &target),
        )))?;

        let total_size = res.content_length().ok_or(Error::new(
            ErrorKind::Other,
            format!("Failed to get content length from '{}'", &target),
        ))?;

        let dest_file = package::place_cached_file(cache_dir, &info.file)?;
        let mut dest = fs::File::create(dest_file)?;
        let mut downloaded: u64 = 0;
        let mut stream = res.bytes_stream();
        let mut total_percentage: i64 = 0;

        while let Some(item) = stream.next().await {
            let chunk = item.or(Err(Error::new(
                ErrorKind::Other,
                "Error while downloading file",
            )))?;
            dest.write_all(&chunk).or(Err(Error::new(
                ErrorKind::Other,
                "Error while writing to file",
            )))?;

            let new = min(downloaded + (chunk.len() as u64), total_size);
            downloaded = new;
            let percentage = ((downloaded as f64 / total_size as f64) * 100_f64) as i64;

            if percentage != total_percentage {
                info!(
                    "download {}%: {} out of {}",
                    percentage, downloaded, total_size
                );

                let status_obj = StatusObj { label: None, progress: Some(percentage) };
                let status_str = serde_json::to_string(&status_obj).unwrap();
                sender.send(status_str).unwrap();

                total_percentage = percentage;
            }
        }

        Ok(())
    }
}
