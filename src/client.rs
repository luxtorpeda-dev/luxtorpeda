extern crate reqwest;

use futures_util::StreamExt;
use gdnative::prelude::*;
use log::{error, info};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::cmp::min;
use std::env;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Write;
use std::io::{Error, ErrorKind};
use std::sync::mpsc::channel;
use tokio::runtime::Runtime;

use crate::command;
use crate::package;
use crate::package::ChoiceInfo;
use crate::user_env;

#[derive(NativeClass)]
#[inherit(Node)]
// register_with attribute can be used to specify custom register function for node signals and properties
#[register_with(Self::register_signals)]
pub struct SignalEmitter;

#[methods]
impl SignalEmitter {
    fn register_signals(builder: &ClassBuilder<Self>) {
        builder.signal("choice_picked").done();
        builder.signal("question_confirmed").done();
        builder.signal("clear_default_choice").done();
        builder.signal("controller_detection_change").done();
    }

    fn new(_owner: &Node) -> Self {
        SignalEmitter
    }
}

#[derive(NativeClass)]
#[inherit(Node)]
pub struct LuxClient {
    receiver: std::option::Option<std::sync::mpsc::Receiver<String>>,
    last_downloads: std::option::Option<Vec<package::PackageInfo>>,
    last_choice: std::option::Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct StatusObj {
    pub label: std::option::Option<String>,
    pub progress: std::option::Option<i64>,
    pub complete: bool,
    pub log_line: std::option::Option<String>,
    pub error: std::option::Option<String>,
    pub prompt_items: std::option::Option<PromptItemsData>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PromptRequestData {
    pub label: std::option::Option<String>,
    pub prompt_type: String,
    pub title: String,
    pub prompt_id: String,
    pub rich_text: std::option::Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PromptItemsData {
    pub prompt_items: Vec<PromptRequestData>,
    pub prompt_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ChoiceData {
    engine_choice: std::option::Option<String>,
    default_engine_choice: std::option::Option<String>,
}

#[methods]
impl LuxClient {
    fn new(_base: &Node) -> Self {
        LuxClient {
            receiver: None,
            last_downloads: None,
            last_choice: None,
        }
    }

    fn show_error(&mut self, base: &Node, error: std::io::Error) {
        let status_obj = StatusObj {
            label: None,
            progress: None,
            complete: false,
            log_line: None,
            error: Some(error.to_string()),
            prompt_items: None,
        };
        let status_str = serde_json::to_string(&status_obj).unwrap();
        let emitter = &mut base.get_node("Container/Progress").unwrap();
        let emitter = unsafe { emitter.assume_safe() };
        emitter.emit_signal("progress_change", &[Variant::new(status_str)]);
    }

    #[method]
    fn _ready(&mut self, #[base] base: TRef<Node>) {
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

        emitter
            .connect(
                "question_confirmed",
                base,
                "question_confirmed",
                VariantArray::new_shared(),
                0,
            )
            .unwrap();

        emitter
            .connect(
                "clear_default_choice",
                base,
                "clear_default_choice",
                VariantArray::new_shared(),
                0,
            )
            .unwrap();

        emitter
            .connect(
                "controller_detection_change",
                base,
                "controller_detection_change",
                VariantArray::new_shared(),
                0,
            )
            .unwrap();

        match self.init(&base) {
            Ok(()) => {}
            Err(err) => {
                error!("init err: {:?}", err);
                self.show_error(&base, err);
            }
        };
    }

    fn init(&mut self, base: &Node) -> io::Result<()> {
        let app_id = user_env::steam_app_id();
        let env_args: Vec<String> = env::args().collect();
        let args: Vec<&str> = env_args.iter().map(|a| a.as_str()).collect();

        info!("luxtorpeda version: {}", env!("CARGO_PKG_VERSION"));
        info!("steam_app_id: {:?}", &app_id);
        info!("original command: {:?}", args);
        info!("working dir: {:?}", env::current_dir());
        info!("tool dir: {:?}", user_env::tool_dir());

        match command::main() {
            Ok(()) => {}
            Err(err) => {
                return Err(err);
            }
        };

        match package::update_packages_json() {
            Ok(()) => {}
            Err(err) => {
                return Err(err);
            }
        };

        match self.ask_for_engine_choice(app_id.as_str(), base) {
            Ok(()) => {}
            Err(err) => {
                return Err(err);
            }
        };

        Ok(())
    }

    #[method]
    fn _physics_process(&mut self, #[base] base: TRef<Node>, _delta: f64) {
        if let Some(receiver) = &self.receiver {
            if let Ok(new_data) = receiver.try_recv() {
                let emitter = &mut base.get_node("Container/Progress").unwrap();
                let emitter = unsafe { emitter.assume_safe() };
                emitter.emit_signal("progress_change", &[Variant::new(&new_data)]);

                if new_data.contains("\"complete\":true") {
                    self.run_game(false);
                }
            }
        }
    }

    fn ask_for_engine_choice(&mut self, app_id: &str, owner: &Node) -> io::Result<()> {
        let game_info = match package::get_game_info(app_id) {
            Ok(game_info) => game_info,
            Err(err) => {
                return Err(err);
            }
        };

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
                        let controller_supported_manual = engines[engine_name_clone_clone_four]
                            ["controllerSupportedManualGame"]
                            == true;

                        if controller_not_supported {
                            choice_info
                                .notices
                                .push("Engine Does Not Have Native Controller Support".to_string());
                        } else if controller_supported
                            && game_info["controllerSteamDefault"] == true
                        {
                            choice_info.notices.push(
                                "Engine Has Native Controller Support And Works Out of the Box"
                                    .to_string(),
                            );
                        } else if controller_supported_manual
                            && game_info["controllerSteamDefault"] == true
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
                        && (game_info["cloudSupported"].is_null()
                            || game_info["cloudSupported"] != true)
                    {
                        choice_info
                            .notices
                            .push("Game Has Cloud Saves But Unknown Status".to_string());
                    } else if game_info["cloudAvailable"] == true
                        && game_info["cloudSupported"] == true
                    {
                        choice_info
                            .notices
                            .push("Cloud Saves Supported".to_string());
                    } else if game_info["cloudAvailable"] == true && game_info["cloudIssue"] == true
                    {
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

            let check_default_choice_file_path =
                package::place_config_file(app_id, "default_engine_choice.txt")?;
            if check_default_choice_file_path.exists() {
                info!("show choice. found default choice.");
                let default_engine_choice_str = fs::read_to_string(check_default_choice_file_path)?;
                info!(
                    "show choice. found default choice. choice is {:?}",
                    default_engine_choice_str
                );

                let mut should_show_confirm = true;

                let config_json_file = user_env::tool_dir().join("config.json");
                let config_json_str = fs::read_to_string(config_json_file)?;
                let config_parsed = json::parse(&config_json_str).unwrap();

                if !config_parsed["disable_default_confirm"].is_null() {
                    let disable_default_confirm = &config_parsed["disable_default_confirm"];
                    if disable_default_confirm == true {
                        info!("show choice. disabling default confirm because of config");
                        should_show_confirm = false;
                    }
                }

                match env::var(package::LUX_DISABLE_DEFAULT_CONFIRM) {
                    Ok(val) => {
                        if val == "1" {
                            info!("show choice. disabling default confirm because of env");
                            should_show_confirm = false;
                        } else if val == "0" {
                            info!("show choice. enabling default confirm because of env");
                            should_show_confirm = true;
                        }
                    }
                    Err(err) => {
                        info!("LUX_DISABLE_DEFAULT_CONFIRM not found: {}", err);
                    }
                }

                if should_show_confirm {
                    let prompt_request = PromptRequestData {
                        label: Some(default_engine_choice_str),
                        prompt_type: "default_choice".to_string(),
                        title: "Default Choice Confirmation".to_string(),
                        prompt_id: "defaultchoiceconfirm".to_string(),
                        rich_text: None,
                    };
                    let prompt_request_str = serde_json::to_string(&prompt_request).unwrap();

                    let emitter = &mut owner.get_node("Container/Prompt").unwrap();
                    let emitter = unsafe { emitter.assume_safe() };
                    emitter.emit_signal("show_prompt", &[Variant::new(prompt_request_str)]);
                } else {
                    let choice_obj = ChoiceData {
                        engine_choice: Some(default_engine_choice_str.to_string()),
                        default_engine_choice: Some(default_engine_choice_str),
                    };
                    let choice_str = serde_json::to_string(&choice_obj).unwrap();
                    self.choice_picked(owner, Variant::new(choice_str));
                }
            } else {
                let choices_str = serde_json::to_string(&choices).unwrap();
                let emitter = &mut owner.get_node("Container/Choices").unwrap();
                let emitter = unsafe { emitter.assume_safe() };
                emitter.emit_signal("choices_found", &[Variant::new(choices_str)]);
            }
        } else {
            let downloads = package::json_to_downloads(app_id, &game_info).unwrap();
            self.last_downloads = Some(downloads);
            self.choice_picked(owner, Variant::new("".to_string()));
        }

        Ok(())
    }

    #[method]
    fn controller_detection_change(&mut self, #[base] _owner: &Node, data: Variant) {
        let data_str = data.try_to::<String>().unwrap();
        info!("controller_detection_change: {}", data_str);
        user_env::set_controller_var(&data_str);
    }

    #[method]
    fn choice_picked(&mut self, #[base] owner: &Node, data: Variant) {
        let app_id = user_env::steam_app_id();
        let mut game_info = match package::get_game_info(app_id.as_str()) {
            Ok(game_info) => game_info,
            Err(err) => {
                self.show_error(owner, err);
                return;
            }
        };

        let emitter = &mut owner.get_node("Container/Progress").unwrap();
        let emitter = unsafe { emitter.assume_safe() };
        emitter.emit_signal("show_progress", &[Variant::new("")]);

        let data_str = data.try_to::<String>().unwrap();

        if !data_str.is_empty() {
            let choice_obj: ChoiceData = serde_json::from_str(&data_str).unwrap();

            if let Some(engine_choice) = choice_obj.engine_choice {
                info!("picked for engine_choice: {}", engine_choice);

                if let Some(default_choice) = choice_obj.default_engine_choice {
                    info!("default engine choice requested for {}", default_choice);
                    let default_choice_file_path =
                        package::place_config_file(&app_id, "default_engine_choice.txt").unwrap();
                    let mut default_choice_file = File::create(default_choice_file_path).unwrap();
                    default_choice_file
                        .write_all(default_choice.as_bytes())
                        .unwrap();
                }

                self.last_choice = Some(engine_choice.clone());

                match package::convert_game_info_with_choice(engine_choice, &mut game_info) {
                    Ok(()) => {
                        info!("engine choice complete");
                    }
                    Err(err) => {
                        error!("convert_game_info_with_choice err: {:?}", err);
                        self.show_error(owner, err);
                        return;
                    }
                };
            }
        }

        if !game_info["app_ids_deps"].is_null() {
            match package::get_app_id_deps_paths(&game_info["app_ids_deps"]) {
                Some(()) => {
                    info!("download_all. get_app_id_deps_paths completed");
                }
                None => {
                    info!("download_all. warning: get_app_id_deps_paths not completed");
                }
            }
        }

        if game_info["download"].is_null() {
            info!("skipping downloads (no urls defined for this package)");
            self.run_game(false);
            return;
        }

        let downloads = package::json_to_downloads(app_id.as_str(), &game_info).unwrap();

        if downloads.is_empty() {
            info!("Downloads is empty");
            self.run_game(false);
            return;
        }

        let mut dialog_message = String::new();

        let mut engine_name = game_info["name"].to_string();
        if !game_info["engine_name"].is_null() {
            engine_name = game_info["engine_name"].to_string();
        }

        let engines_option = package::get_engines_info();
        let engine_name_clone = engine_name.clone();
        let engine_name_clone2 = engine_name.clone();
        if let Some((ref engines, ref _notice_map)) = engines_option {
            if !engines[engine_name_clone].is_null() {
                for entry in engines[engine_name]["notices"].members() {
                    let engine_name_clone3 = engine_name_clone2.clone();
                    if !entry["key"].is_null() {
                        if entry["key"] == "non_free" {
                            dialog_message = std::format!(
                            "This engine uses a non-free engine ({0}). Are you sure you want to continue?",
                            engines[engine_name_clone3]["license"]
                        );
                        } else if entry["key"] == "closed_source" {
                            dialog_message = "This engine uses assets from the closed source release. Are you sure you want to continue?".to_string();
                        }
                    }
                }
            }
        }

        self.last_downloads = Some(downloads);

        if !dialog_message.is_empty() {
            let prompt_request = PromptRequestData {
                label: Some(dialog_message),
                prompt_type: "question".to_string(),
                title: "License Warning".to_string(),
                prompt_id: "confirmlicensedownload".to_string(),
                rich_text: None,
            };
            let prompt_request_str = serde_json::to_string(&prompt_request).unwrap();

            let emitter = &mut owner.get_node("Container/Prompt").unwrap();
            let emitter = unsafe { emitter.assume_safe() };
            emitter.emit_signal("show_prompt", &[Variant::new(prompt_request_str)]);
        } else {
            self.process_download();
        }
    }

    #[method]
    fn question_confirmed(&mut self, #[base] owner: &Node, data: Variant) {
        let mode_id = data.try_to::<String>().unwrap();
        info!("question_confirmed with mode: {}", mode_id);

        if mode_id == "confirmlicensedownload" {
            let emitter = &mut owner.get_node("Container/Progress").unwrap();
            let emitter = unsafe { emitter.assume_safe() };
            emitter.emit_signal("show_progress", &[Variant::new("")]);

            self.process_download();
        } else if mode_id.contains("dialogentryconfirm") {
            let mode_split = mode_id.split("%%");
            let mode_items = mode_split.collect::<Vec<&str>>();
            if mode_items.len() == 3 {
                let key = mode_items[1];
                let text_input = mode_items[2];

                info!(
                    "found dialog entry response for key: {} with value: {}",
                    key, text_input
                );
                if !key.is_empty() {
                    env::set_var(std::format!("DIALOGRESPONSE_{}", key), text_input);
                }
            }
        } else if mode_id.contains("allprompts") {
            let mode_split = mode_id.split("allprompts");
            let mode_items = mode_split.collect::<Vec<&str>>();
            if mode_items.len() == 2 {
                let mode_id = mode_items[1];
                if mode_id == "setup" {
                    let emitter = &mut owner.get_node("Container/Progress").unwrap();
                    let emitter = unsafe { emitter.assume_safe() };
                    emitter.emit_signal("show_progress", &[Variant::new("")]);

                    self.run_game(true);
                }
            }
        } else if mode_id.contains("cancel%%") {
            let mode_split = mode_id.split("cancel%%");
            let mode_items = mode_split.collect::<Vec<&str>>();
            if mode_items.len() == 2 {
                let mode_id = mode_items[1];
                if mode_id == "download" {
                    if let Some(last_downloads) = &self.last_downloads {
                        for info in last_downloads.iter() {
                            let app_id = user_env::steam_app_id();
                            let mut cache_dir = app_id;
                            if info.cache_by_name {
                                cache_dir = info.name.to_string();
                            }

                            let dest_file =
                                package::place_cached_file(&cache_dir, &info.file).unwrap();
                            if dest_file.exists() {
                                info!("download cancel, removing file: {:?}", dest_file);
                                fs::remove_file(dest_file).unwrap();
                            }
                        }
                    }
                }
            }
            std::process::exit(0);
        }
    }

    #[method]
    fn clear_default_choice(&mut self, #[base] owner: &Node, _data: Variant) {
        let app_id = user_env::steam_app_id();
        let config_path = package::path_to_config();
        let folder_path = config_path.join(&app_id);
        match fs::remove_dir_all(folder_path) {
            Ok(()) => {
                info!("clear config done");
            }
            Err(err) => {
                error!("clear config. err: {:?}", err);
            }
        }

        match self.ask_for_engine_choice(app_id.as_str(), owner) {
            Ok(()) => {}
            Err(err) => {
                error!("clear_default_choice ask err: {:?}", err);
                self.show_error(owner, err);
            }
        };
    }

    fn process_download(&mut self) {
        let app_id = user_env::steam_app_id();

        if let Some(last_downloads) = self.last_downloads.as_mut() {
            let downloads = last_downloads.clone();
            let (sender, receiver) = channel();
            self.receiver = Some(receiver);

            std::thread::spawn(move || {
                let client = reqwest::Client::new();

                let mut found_error = false;

                for (i, info) in downloads.iter().enumerate() {
                    let app_id = app_id.to_string();
                    info!("starting download on: {} {}", i, info.name.clone());

                    let label_str = std::format!(
                        "Downloading {}/{} - {}",
                        i + 1,
                        downloads.len(),
                        info.name.clone()
                    );

                    let status_obj = StatusObj {
                        label: Some(label_str),
                        progress: None,
                        complete: false,
                        log_line: None,
                        error: None,
                        prompt_items: None,
                    };
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
                            let error_str =
                                std::format!("Download of {} Error: {}", info.name.clone(), err);
                            error!("{}", error_str);

                            let status_obj = StatusObj {
                                label: None,
                                progress: None,
                                complete: false,
                                log_line: None,
                                error: Some(error_str),
                                prompt_items: None,
                            };
                            let status_str = serde_json::to_string(&status_obj).unwrap();
                            sender.send(status_str).unwrap();

                            let mut cache_dir = app_id;
                            if info.cache_by_name {
                                cache_dir = info.name.clone();
                            }
                            let dest_file =
                                package::place_cached_file(&cache_dir, &info.file).unwrap();
                            if dest_file.exists() {
                                fs::remove_file(dest_file).unwrap();
                            }

                            found_error = true;
                        }
                    };

                    if found_error {
                        break;
                    }
                }

                if !found_error {
                    let status_obj = StatusObj {
                        label: None,
                        progress: None,
                        complete: true,
                        log_line: None,
                        error: None,
                        prompt_items: None,
                    };
                    let status_str = serde_json::to_string(&status_obj).unwrap();
                    sender.send(status_str).unwrap();
                }
            });
        }
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

        let res = client.get(&target).send().await.map_err(|_| {
            Error::new(
                ErrorKind::Other,
                format!("Failed to GET from '{}'", &target),
            )
        })?;

        let total_size = res.content_length().ok_or_else(|| {
            Error::new(
                ErrorKind::Other,
                format!("Failed to get content length from '{}'", &target),
            )
        })?;

        let dest_file = package::place_cached_file(cache_dir, &info.file)?;
        let mut dest = fs::File::create(dest_file)?;
        let mut downloaded: u64 = 0;
        let mut stream = res.bytes_stream();
        let mut total_percentage: i64 = 0;

        while let Some(item) = stream.next().await {
            let chunk =
                item.map_err(|_| Error::new(ErrorKind::Other, "Error while downloading file"))?;
            dest.write_all(&chunk)
                .map_err(|_| Error::new(ErrorKind::Other, "Error while writing to file"))?;

            let new = min(downloaded + (chunk.len() as u64), total_size);
            downloaded = new;
            let percentage = ((downloaded as f64 / total_size as f64) * 100_f64) as i64;

            if percentage != total_percentage {
                info!(
                    "download {}%: {} out of {}",
                    percentage, downloaded, total_size
                );

                let status_obj = StatusObj {
                    label: None,
                    progress: Some(percentage),
                    complete: false,
                    log_line: None,
                    error: None,
                    prompt_items: None,
                };
                let status_str = serde_json::to_string(&status_obj).unwrap();
                sender.send(status_str).unwrap();

                total_percentage = percentage;
            }
        }

        Ok(())
    }

    fn run_game(&mut self, after_setup_question_mode: bool) {
        if !user_env::manual_download_app_id().is_empty() {
            std::process::exit(0);
        }

        let mut engine_choice = String::new();

        if let Some(choice) = &self.last_choice {
            engine_choice = choice.to_string();
        }

        let (sender, receiver) = channel();
        self.receiver = Some(receiver);

        std::thread::spawn(move || {
            let env_args: Vec<String> = env::args().collect();
            let args: Vec<&str> = env_args.iter().map(|a| a.as_str()).collect();
            let cmd_args = &args[2..];

            let sender_err = sender.clone();

            let game_info =
                match command::run(cmd_args, engine_choice, &sender, after_setup_question_mode) {
                    Ok(game_info) => game_info,
                    Err(err) => {
                        error!("command::run err: {:?}", err);

                        let status_obj = StatusObj {
                            label: None,
                            progress: None,
                            complete: false,
                            log_line: None,
                            error: Some(err.to_string()),
                            prompt_items: None,
                        };
                        let status_str = serde_json::to_string(&status_obj).unwrap();
                        sender_err.send(status_str).unwrap();

                        return;
                    }
                };

            if !game_info["setup"].is_null()
                && !after_setup_question_mode
                && !package::is_setup_complete(&game_info["setup"])
            {
                match command::process_setup_details(&game_info) {
                    Ok(setup_details) => {
                        info!("setup details ready: {:?}", setup_details);

                        if !setup_details.is_empty() {
                            let prompt_items = PromptItemsData {
                                prompt_items: setup_details,
                                prompt_id: "allpromptssetup".to_string(),
                            };
                            let status_obj = StatusObj {
                                label: None,
                                progress: None,
                                complete: false,
                                log_line: Some("Processing setup items".to_string()),
                                error: None,
                                prompt_items: Some(prompt_items),
                            };
                            let status_str = serde_json::to_string(&status_obj).unwrap();
                            sender_err.send(status_str).unwrap();

                            return;
                        } else {
                            match command::run_setup(&game_info, &sender) {
                                Ok(()) => {}
                                Err(err) => {
                                    error!("command::run_setup err: {:?}", err);

                                    let status_obj = StatusObj {
                                        label: None,
                                        progress: None,
                                        complete: false,
                                        log_line: None,
                                        error: Some(err.to_string()),
                                        prompt_items: None,
                                    };
                                    let status_str = serde_json::to_string(&status_obj).unwrap();
                                    sender_err.send(status_str).unwrap();

                                    return;
                                }
                            }
                        }
                    }
                    Err(err) => {
                        error!("command::process_setup_details err: {:?}", err);

                        let status_obj = StatusObj {
                            label: None,
                            progress: None,
                            complete: false,
                            log_line: None,
                            error: Some(err.to_string()),
                            prompt_items: None,
                        };
                        let status_str = serde_json::to_string(&status_obj).unwrap();
                        sender_err.send(status_str).unwrap();

                        return;
                    }
                }
            }

            if after_setup_question_mode {
                match command::run_setup(&game_info, &sender) {
                    Ok(()) => {}
                    Err(err) => {
                        error!("command::run_setup err: {:?}", err);

                        let status_obj = StatusObj {
                            label: None,
                            progress: None,
                            complete: false,
                            log_line: None,
                            error: Some(err.to_string()),
                            prompt_items: None,
                        };
                        let status_str = serde_json::to_string(&status_obj).unwrap();
                        sender_err.send(status_str).unwrap();

                        return;
                    }
                }
            }

            match command::run_wrapper(cmd_args, &game_info, &sender_err) {
                Ok(()) => {}
                Err(err) => {
                    error!("command::run_wrapper err: {:?}", err);

                    let status_obj = StatusObj {
                        label: None,
                        progress: None,
                        complete: false,
                        log_line: None,
                        error: Some(err.to_string()),
                        prompt_items: None,
                    };
                    let status_str = serde_json::to_string(&status_obj).unwrap();
                    sender_err.send(status_str).unwrap();
                }
            };
        });
    }
}
