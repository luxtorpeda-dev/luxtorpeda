extern crate reqwest;

use futures_util::StreamExt;
use log::{error, info};
use reqwest::redirect::Policy;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::cmp::min;
use std::env;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Error;
use std::io::Write;
use std::sync::mpsc::channel;
use tokio::runtime::Runtime;

use godot::builtin::Variant;
use godot::classes::{Engine, Node, Os};
use godot::prelude::*;

use crate::command;
use crate::config;
use crate::package;
use crate::package_metadata;
use crate::proton_handler::{find_tool_by_name, list_proton_tools, Tool};
use crate::user_env;

#[derive(GodotClass)]
#[class(base=Node)]
pub struct LuxClient {
    receiver: std::option::Option<std::sync::mpsc::Receiver<String>>,
    last_downloads: std::option::Option<Vec<package_metadata::DownloadItem>>,
    last_choice: std::option::Option<String>,
    proton_tools: std::option::Option<Vec<Tool>>,
    base: Base<Node>,
}

#[derive(Default, Serialize, Deserialize, Debug)]
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

#[godot_api]
impl INode for LuxClient {
    fn init(base: Base<Node>) -> Self {
        Self {
            receiver: None,
            last_downloads: None,
            last_choice: None,
            proton_tools: None,
            base,
        }
    }

    fn ready(&mut self) {
        match self.init() {
            Ok(()) => {}
            Err(err) => {
                error!("init err: {:?}", err);
                self.show_error(err);
            }
        };
    }

    fn physics_process(&mut self, _delta: f64) {
        if let Some(receiver) = &self.receiver {
            if let Ok(new_data) = receiver.try_recv() {
                if new_data == "quit_client" {
                    let mut tree: Gd<SceneTree> =
                        Engine::singleton().get_main_loop().unwrap().cast();
                    let args: &[Variant] = &[];
                    tree.call_deferred("quit", args);
                    return;
                }
                self.emit_signal(
                    "Container/Progress",
                    "progress_change",
                    &new_data.to_string(),
                );

                if new_data.contains("\"complete\":true") {
                    self.run_game(false);
                }
            }
        }
    }
}

#[godot_api]
impl LuxClient {
    fn show_error(&mut self, error: std::io::Error) {
        let status_obj = StatusObj {
            error: Some(error.to_string()),
            ..Default::default()
        };
        let status_str = serde_json::to_string(&status_obj).unwrap();
        self.emit_signal(
            "Container/Progress",
            "progress_change",
            &status_str.to_string(),
        );
    }

    fn emit_signal(&mut self, path: &str, name: &str, value: &str) {
        if let Some(parent) = &mut self.base().get_parent() {
            let mut emitter = parent.get_node_as::<Node>(&NodePath::from(path));
            emitter.emit_signal(name, &[Variant::from(value)]);
        } else {
            error!("emit_signal parent not found for {}", path);
        }
    }

    fn init(&mut self) -> io::Result<()> {
        let app_id = user_env::steam_app_id();
        let env_args: Vec<String> = env::args().collect();
        let args: Vec<&str> = env_args.iter().map(|a| a.as_str()).collect();

        let running_in_editor = !Os::singleton().has_feature("template");

        match command::main(running_in_editor) {
            Ok(()) => {}
            Err(err) => {
                return Err(err);
            }
        };

        let lux_pwd = user_env::lux_pwd().unwrap();
        env::set_current_dir(lux_pwd).unwrap();

        info!("luxtorpeda version: {}", env!("CARGO_PKG_VERSION"));
        info!("steam_app_id: {:?}", &app_id);
        info!("original command: {:?}", args);
        info!("working dir: {:?}", env::current_dir());
        info!("tool dir: {:?}", user_env::tool_dir());

        match package_metadata::PackageMetadata::update_packages_json() {
            Ok(()) => {}
            Err(err) => {
                return Err(err);
            }
        };

        match self.ask_for_engine_choice(app_id.as_str()) {
            Ok(()) => {}
            Err(err) => {
                return Err(err);
            }
        };

        Ok(())
    }

    fn show_proton(&mut self) -> io::Result<()> {
        let mut choices: Vec<package_metadata::SimpleEngineChoice> = vec![];

        if let Some(steam_path) = user_env::steam_install_path() {
            let tools = list_proton_tools(&steam_path).expect("Cannot find Proton tools");
            self.proton_tools = Some(tools.clone());

            for tool in tools {
                choices.insert(
                    0,
                    package_metadata::SimpleEngineChoice {
                        name: tool.display_name,
                        notices: Vec::new(),
                    },
                );
            }

            let choices_str = serde_json::to_string(&choices).unwrap();

            self.emit_signal(
                "Container/Choices",
                "choices_found",
                &choices_str.to_string(),
            );
        }

        Ok(())
    }

    fn ask_for_engine_choice(&mut self, app_id: &str) -> io::Result<()> {
        let mut game_info = match package::get_game_info(app_id) {
            Ok(game_info) => game_info,
            Err(err) => {
                return Err(err);
            }
        };

        if game_info.choices.is_some() {
            let choices = game_info.choices_with_notices();

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
                let config = config::Config::from_config_file();

                if config.disable_default_confirm {
                    info!("show choice. disabling default confirm because of config");
                    should_show_confirm = false;
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

                    self.emit_signal(
                        "Container/Prompt",
                        "show_prompt",
                        &prompt_request_str.to_string(),
                    );
                } else {
                    let choice_obj = ChoiceData {
                        engine_choice: Some(default_engine_choice_str.to_string()),
                        default_engine_choice: Some(default_engine_choice_str),
                    };
                    let choice_str = serde_json::to_string(&choice_obj).unwrap();
                    self.choice_picked(Variant::from(choice_str));
                }
            } else {
                let choices_str = serde_json::to_string(&choices).unwrap();
                self.emit_signal(
                    "Container/Choices",
                    "choices_found",
                    &choices_str.to_string(),
                );
            }
        } else {
            let downloads = package::json_to_downloads(app_id, &game_info).unwrap();
            self.last_downloads = Some(downloads);
            self.choice_picked(Variant::from("".to_string()));
        }

        Ok(())
    }

    #[func]
    fn controller_detection_change(&mut self, data: Variant) {
        let data_str = data.try_to::<String>().unwrap();
        info!("controller_detection_change: {}", data_str);
        user_env::set_controller_var(&data_str);
    }

    #[func]
    fn choice_picked(&mut self, data: Variant) {
        let app_id = user_env::steam_app_id();
        let mut game_info = match package::get_game_info(app_id.as_str()) {
            Ok(game_info) => game_info,
            Err(err) => {
                self.show_error(err);
                return;
            }
        };

        self.emit_signal("Container/Progress", "show_progress", "");

        let data_str = data.try_to::<String>().unwrap();

        if !data_str.is_empty() {
            let choice_obj: ChoiceData = serde_json::from_str(&data_str).unwrap();
            let is_in_proton_selection = self.last_choice == Some("Choose Proton".to_string());

            if let Some(engine_choice) = choice_obj.engine_choice {
                info!("picked for engine_choice: {}", engine_choice);

                if let Some(default_choice) = choice_obj.default_engine_choice {
                    if !is_in_proton_selection {
                        info!("default engine choice requested for {}", default_choice);
                        let default_choice_file_path =
                            package::place_config_file(&app_id, "default_engine_choice.txt")
                                .unwrap();
                        let mut default_choice_file =
                            File::create(default_choice_file_path).unwrap();
                        default_choice_file
                            .write_all(default_choice.as_bytes())
                            .unwrap();
                    } else {
                        info!("default proton version requested for {}", default_choice);
                        let xdg_dirs = xdg::BaseDirectories::with_prefix("luxtorpeda");
                        let default_proton_file_path = xdg_dirs
                            .place_config_file("default_proton_choice.txt")
                            .unwrap();
                        let mut default_proton_file =
                            File::create(default_proton_file_path).unwrap();

                        if let Some(proton_tools) = &self.proton_tools {
                            let Some(proton) = find_tool_by_name(&proton_tools, &default_choice)
                            else {
                                return;
                            };

                            default_proton_file
                                .write_all(proton.alias.as_bytes())
                                .unwrap();
                        }
                        let _ = self.ask_for_engine_choice(app_id.as_str());
                        self.last_choice = Some("".to_string());

                        return;
                    }
                }

                if is_in_proton_selection {
                    info!(
                        "user picked proton version {} for {}",
                        engine_choice, app_id
                    );
                    let proton_choice_file_path =
                        package::place_config_file(&app_id, "proton_choice.txt").unwrap();
                    let mut proton_choice_file = File::create(proton_choice_file_path).unwrap();

                    if let Some(proton_tools) = &self.proton_tools {
                        let Some(proton) = find_tool_by_name(&proton_tools, &engine_choice) else {
                            return;
                        };

                        proton_choice_file
                            .write_all(proton.alias.as_bytes())
                            .unwrap();
                    }

                    let _ = self.ask_for_engine_choice(app_id.as_str());
                    self.last_choice = Some("".to_string());
                    return;
                }

                self.last_choice = Some(engine_choice.clone());

                if engine_choice == "Choose Proton" {
                    let _ = self.show_proton();
                    return;
                }

                match package::convert_game_info_with_choice(engine_choice, &mut game_info) {
                    Ok(()) => {
                        info!("engine choice complete");
                    }
                    Err(err) => {
                        error!("convert_game_info_with_choice err: {:?}", err);
                        self.show_error(err);
                        return;
                    }
                };
            }
        }

        let downloads = package::json_to_downloads(app_id.as_str(), &game_info).unwrap();

        if downloads.is_empty() {
            info!("Downloads is empty");
            self.run_game(false);
            return;
        }

        self.last_downloads = Some(downloads);

        if let Some(dialog_message) = game_info.find_license_dialog_message() {
            let prompt_request = PromptRequestData {
                label: Some(dialog_message),
                prompt_type: "question".to_string(),
                title: "License Warning".to_string(),
                prompt_id: "confirmlicensedownload".to_string(),
                rich_text: None,
            };
            let prompt_request_str = serde_json::to_string(&prompt_request).unwrap();

            self.emit_signal(
                "Container/Prompt",
                "show_prompt",
                &prompt_request_str.to_string(),
            );
        } else {
            self.process_download();
        }
    }

    #[func]
    fn question_confirmed(&mut self, data: Variant) {
        let mode_id = data.try_to::<String>().unwrap();
        info!("question_confirmed with mode: {}", mode_id);

        if mode_id == "confirmlicensedownload" {
            self.emit_signal("Container/Progress", "show_progress", "");
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
                    self.emit_signal("Container/Progress", "show_progress", "");
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
            Self::exit_client();
        }
    }

    #[func]
    fn clear_default_choice(&mut self) {
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

        match self.ask_for_engine_choice(app_id.as_str()) {
            Ok(()) => {}
            Err(err) => {
                error!("clear_default_choice ask err: {:?}", err);
                self.show_error(err);
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
                let client = Client::builder()
                    .redirect(Policy::limited(10))
                    .build()
                    .unwrap();

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
                        ..Default::default()
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
                                error: Some(error_str),
                                ..Default::default()
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
                        complete: true,
                        ..Default::default()
                    };
                    let status_str = serde_json::to_string(&status_obj).unwrap();
                    sender.send(status_str).unwrap();
                }
            });
        }
    }

    async fn download(
        app_id: &str,
        info: &package_metadata::DownloadItem,
        sender: std::sync::mpsc::Sender<String>,
        client: &Client,
    ) -> io::Result<()> {
        let mut target = info.url.clone() + &info.file;

        if let Some(suffix) = &info.url_suffix {
            target.push_str(suffix);
        }

        let mut cache_dir = app_id;
        if info.cache_by_name {
            cache_dir = &info.name;
        }

        info!("download target: {:?}", target);

        let res = client
            .get(&target)
            .send()
            .await
            .map_err(|_| Error::other(format!("Failed to GET from '{}'", &target)))?;

        let total_size = res.content_length().unwrap_or(0);

        let dest_file = package::place_cached_file(cache_dir, &info.file)?;
        let mut dest = fs::File::create(dest_file)?;
        let mut downloaded: u64 = 0;
        let mut stream = res.bytes_stream();
        let mut total_percentage: i64 = 0;

        if total_size == 0 {
            let total_size_message =
                std::format!("Downloading {} without a known file size", &info.name);
            let status_obj = StatusObj {
                log_line: Some(total_size_message),
                ..Default::default()
            };
            let status_str = serde_json::to_string(&status_obj).unwrap();
            sender.send(status_str).unwrap();
        }

        while let Some(item) = stream.next().await {
            let chunk = item.map_err(|_| Error::other("Error while downloading file"))?;
            dest.write_all(&chunk)
                .map_err(|_| Error::other("Error while writing to file"))?;

            if total_size > 0 {
                let new = min(downloaded + (chunk.len() as u64), total_size);
                downloaded = new;
                let percentage = ((downloaded as f64 / total_size as f64) * 100_f64) as i64;

                if percentage != total_percentage {
                    info!(
                        "download {}%: {} out of {}",
                        percentage, downloaded, total_size
                    );

                    let status_obj = StatusObj {
                        progress: Some(percentage),
                        ..Default::default()
                    };
                    let status_str = serde_json::to_string(&status_obj).unwrap();
                    sender.send(status_str).unwrap();

                    total_percentage = percentage;
                }
            } else {
                info!("downloaded without known total size: {}", downloaded);
            }
        }

        Ok(())
    }

    fn exit_client() {
        if let Some(main_loop) = Engine::singleton().get_main_loop() {
            let mut tree: Gd<SceneTree> = main_loop.cast();
            let args: &[Variant] = &[];
            tree.call_deferred("quit", args);
        }
    }

    fn run_game(&mut self, after_setup_question_mode: bool) {
        if !user_env::manual_download_app_id().is_empty() {
            Self::exit_client();
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
                            error: Some(err.to_string()),
                            ..Default::default()
                        };
                        let status_str = serde_json::to_string(&status_obj).unwrap();
                        sender_err.send(status_str).unwrap();

                        return;
                    }
                };

            if let Some(app_ids_deps) = &game_info.app_ids_deps {
                let status_obj = StatusObj {
                    log_line: Some("Checking for steam app dependency paths".to_string()),
                    ..Default::default()
                };
                let status_str = serde_json::to_string(&status_obj).unwrap();
                sender_err.send(status_str).unwrap();

                let sender_paths = sender_err.clone();

                match package::get_app_id_deps_paths(app_ids_deps, false, &sender_paths) {
                    Ok(()) => {
                        info!("run_game. get_app_id_deps_paths completed");
                    }
                    Err(err) => {
                        let error_message = std::format!(
                            "run_game. error: get_app_id_deps_paths not completed, error: {:?}",
                            err
                        );

                        error!("{}", error_message);

                        let status_obj = StatusObj {
                            error: Some(error_message),
                            ..Default::default()
                        };
                        let status_str = serde_json::to_string(&status_obj).unwrap();
                        sender_err.send(status_str).unwrap();

                        return;
                    }
                }
            }

            if let Some(setup_info) = &game_info.setup {
                if !after_setup_question_mode && !package::is_setup_complete(setup_info) {
                    match command::process_setup_details(setup_info) {
                        Ok(setup_details) => {
                            info!("setup details ready: {:?}", setup_details);

                            if !setup_details.is_empty() {
                                let prompt_items = PromptItemsData {
                                    prompt_items: setup_details,
                                    prompt_id: "allpromptssetup".to_string(),
                                };
                                let status_obj = StatusObj {
                                    log_line: Some("Processing setup items".to_string()),
                                    prompt_items: Some(prompt_items),
                                    ..Default::default()
                                };
                                let status_str = serde_json::to_string(&status_obj).unwrap();
                                sender_err.send(status_str).unwrap();

                                return;
                            } else {
                                match command::run_setup(setup_info, &sender) {
                                    Ok(()) => {}
                                    Err(err) => {
                                        error!("command::run_setup err: {:?}", err);

                                        let status_obj = StatusObj {
                                            error: Some(err.to_string()),
                                            ..Default::default()
                                        };
                                        let status_str =
                                            serde_json::to_string(&status_obj).unwrap();
                                        sender_err.send(status_str).unwrap();

                                        return;
                                    }
                                }
                            }
                        }
                        Err(err) => {
                            error!("command::process_setup_details err: {:?}", err);

                            let status_obj = StatusObj {
                                error: Some(err.to_string()),
                                ..Default::default()
                            };
                            let status_str = serde_json::to_string(&status_obj).unwrap();
                            sender_err.send(status_str).unwrap();

                            return;
                        }
                    }
                }

                if after_setup_question_mode {
                    match command::run_setup(setup_info, &sender) {
                        Ok(()) => {}
                        Err(err) => {
                            error!("command::run_setup err: {:?}", err);

                            let status_obj = StatusObj {
                                error: Some(err.to_string()),
                                ..Default::default()
                            };
                            let status_str = serde_json::to_string(&status_obj).unwrap();
                            sender_err.send(status_str).unwrap();

                            return;
                        }
                    }
                }
            }

            let steam_input_template_path = std::path::Path::new("steam_input_template.vdf");
            if steam_input_template_path.exists() {
                let app_id = user_env::steam_app_id().parse::<u32>().unwrap();
                package::install_steam_input_template(&app_id, steam_input_template_path);
            }

            match command::run_wrapper(cmd_args, &game_info, &sender_err) {
                Ok(()) => {}
                Err(err) => {
                    error!("command::run_wrapper err: {:?}", err);

                    let status_obj = StatusObj {
                        error: Some(err.to_string()),
                        ..Default::default()
                    };
                    let status_str = serde_json::to_string(&status_obj).unwrap();
                    sender_err.send(status_str).unwrap();
                }
            };
        });
    }
}
