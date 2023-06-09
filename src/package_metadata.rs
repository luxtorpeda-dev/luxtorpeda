use log::{error, info};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::package;

const PACKAGE_METADATA_FILENAME: &str = "packagessniper_v2";

#[derive(Default, Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct PackageMetadata {
    pub games: Vec<Game>,
    pub engines: Vec<Engine>,
    pub default_engine: Game,
}

#[derive(Default, Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct Game {
    pub game_name: String,
    pub engine_name: String,
    pub command: Option<String>,
    pub command_args: Vec<String>,
    pub download_config: Option<Vec<DownloadConfig>>,
    #[serde(alias = "cloudNotAvailable")]
    pub cloud_not_available: bool,
    #[serde(alias = "cloudSupported")]
    pub cloud_supported: bool,
    #[serde(alias = "cloudAvailable")]
    pub cloud_available: bool,
    #[serde(alias = "cloudIssue")]
    pub cloud_issue: bool,
    pub download: Vec<DownloadItem>,
    pub app_id: String,
    pub choices: Option<Vec<EngineChoice>>,
    notices: Option<Vec<Notice>>,
    #[serde(alias = "controllerSteamDefault")]
    pub controller_steam_default: bool,
    pub use_original_command_directory: bool,
    pub app_ids_deps: Option<Vec<u32>>,
    pub setup: Option<Setup>,
    pub commands: Option<Vec<GameCommand>>,
}

#[derive(Default, Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct DownloadItem {
    pub name: String,
    pub url: String,
    pub file: String,
    pub cache_by_name: bool,
}

#[derive(Default, Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct DownloadConfig {
    pub download_name: String,
    pub extract_location: Option<String>,
    pub setup: bool,
    pub strip_prefix: Option<String>,
}

#[derive(Default, Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct EngineChoice {
    pub name: String,
    pub engine_name: Option<String>,
    pub command: Option<String>,
    pub command_args: Vec<String>,
    pub download: Option<Vec<String>>,
    pub download_config: Option<Vec<DownloadConfig>>,
    notices: Option<Vec<Notice>>,
    pub commands: Option<Vec<GameCommand>>,
}

#[derive(Default, Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct SimpleEngineChoice {
    pub name: String,
    notices: Vec<String>,
}

#[derive(Default, Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct GameCommand {
    pub cmd: String,
    pub args: Vec<String>,
    pub command_name: String,
}

#[derive(Default, Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct Setup {
    pub complete_path: String,
    pub command: String,
    pub uninstall_command: Option<String>,
    pub license_path: Option<String>,
    pub dialogs: Option<Vec<SetupDialog>>,
}

#[derive(Default, Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct SetupDialog {
    #[serde(alias = "type")]
    pub dialog_type: String,
    pub title: String,
    pub label: String,
    pub key: String,
}

#[derive(Default, Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct Engine {
    pub engine_link: String,
    pub version: String,
    pub author: String,
    pub author_link: String,
    pub license: String,
    pub license_link: String,
    #[serde(alias = "controllerNotSupported")]
    pub controller_not_supported: bool,
    #[serde(alias = "controllerSupported")]
    pub controller_supported: bool,
    #[serde(alias = "controllerSupportedManualGame")]
    pub controller_supported_manual_game: bool,
    pub engine_name: String,
    notices: Option<Vec<Notice>>,
}

#[derive(Default, Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct Notice {
    pub label: Option<String>,
    pub key: Option<String>,
    pub value: Option<String>,
}

impl PackageMetadata {
    pub fn from_packages_file() -> PackageMetadata {
        let packages_json_file = PackageMetadata::path_to_packages_file();
        if packages_json_file.exists() {
            info!("packages_json_file exists, reading");
            match fs::read_to_string(packages_json_file) {
                Ok(s) => match serde_json::from_str::<PackageMetadata>(&s) {
                    Ok(config) => config,
                    Err(err) => {
                        error!("error parsing packages_json_file: {:?}", err);
                        PackageMetadata::get_packages_from_server()
                    }
                },
                Err(err) => {
                    error!("error reading packages_json_file: {:?}", err);
                    PackageMetadata::get_packages_from_server()
                }
            }
        } else {
            info!("packages_json_file does not exist");
            PackageMetadata::get_packages_from_server()
        }
    }

    pub fn find_game_by_app_id(&self, app_id: &str) -> Option<Game> {
        self.games.iter().find(|x| x.app_id == app_id).cloned()
    }

    pub fn find_engine_by_name(&self, name: &str) -> Option<Engine> {
        self.engines.iter().find(|x| x.engine_name == name).cloned()
    }

    pub fn convert_notice_to_str(notice_item: &Notice) -> String {
        // TODO: fix this!
        let mut notice = String::new();
        notice
    }

    fn get_packages_from_server() -> PackageMetadata {
        // TODO: fix, move code to new function from package to here that package calls once to get download and hash; this function should just call that one
        Default::default()
    }

    fn path_to_packages_file() -> PathBuf {
        let xdg_dirs = xdg::BaseDirectories::new().unwrap();
        let config_home = xdg_dirs.get_cache_home();
        let folder_path = config_home.join("luxtorpeda");
        package::create_dir_or_show_error(&folder_path);
        folder_path.join(format!("{}.json", PACKAGE_METADATA_FILENAME))
    }
}

impl Game {
    pub fn choices_with_notices(&mut self) -> Vec<SimpleEngineChoice> {
        let mut simple_choices: Vec<SimpleEngineChoice> = vec![];

        if let Some(choices) = &self.choices {
            let package_metadata = PackageMetadata::from_packages_file();

            for choice in choices {
                let mut choice_info = SimpleEngineChoice {
                    name: choice.name.clone(),
                    notices: Vec::new(),
                };

                let mut engine_name = &choice.name;
                if let Some(choice_engine_name) = &choice.engine_name {
                    engine_name = choice_engine_name;
                }

                if let Some(engine) =
                    PackageMetadata::find_engine_by_name(&package_metadata, &engine_name)
                {
                    if let Some(engine_notices) = engine.notices {
                        for entry in engine_notices {
                            choice_info
                                .notices
                                .push(PackageMetadata::convert_notice_to_str(&entry));
                        }
                    }

                    let controller_not_supported = engine.controller_not_supported;
                    let controller_supported = engine.controller_supported;
                    let controller_supported_manual = engine.controller_supported_manual_game;

                    if controller_not_supported {
                        choice_info
                            .notices
                            .push("Engine Does Not Have Native Controller Support".to_string());
                    } else if controller_supported && self.controller_steam_default {
                        choice_info.notices.push(
                            "Engine Has Native Controller Support And Works Out of the Box"
                                .to_string(),
                        );
                    } else if controller_supported_manual && self.controller_steam_default {
                        choice_info.notices.push(
                                "Engine Has Native Controller Support But Needs Manual In-Game Settings"
                                    .to_string(),
                            );
                    } else if controller_supported && !self.controller_steam_default {
                        choice_info.notices.push(
                            "Engine Has Native Controller Support But Needs Manual Steam Settings"
                                .to_string(),
                        );
                    }
                }

                if self.cloud_not_available {
                    choice_info
                        .notices
                        .push("Game Does Not Have Cloud Saves".to_string());
                } else if self.cloud_available && !self.cloud_supported {
                    choice_info
                        .notices
                        .push("Game Has Cloud Saves But Unknown Status".to_string());
                } else if self.cloud_available && self.cloud_supported {
                    choice_info
                        .notices
                        .push("Cloud Saves Supported".to_string());
                } else if self.cloud_available && self.cloud_issue {
                    choice_info
                        .notices
                        .push("Cloud Saves Not Supported".to_string());
                }

                if let Some(game_notices) = &self.notices {
                    for entry in game_notices {
                        choice_info
                            .notices
                            .push(PackageMetadata::convert_notice_to_str(&entry));
                    }
                }

                simple_choices.push(choice_info);
            }
        }

        simple_choices
    }

    pub fn find_license_dialog_message(&self) -> Option<String> {
        let package_metadata = PackageMetadata::from_packages_file();
        if let Some(engine) = PackageMetadata::find_engine_by_name(&package_metadata, &self.engine_name)
        {
            if let Some(engine_notices) = engine.notices {
                for entry in engine_notices {
                    if let Some(key) = entry.key {
                        if key == "non_free" {
                            return Some(std::format!(
                            "This engine uses a non-free engine ({0}). Are you sure you want to continue?",
                            engine.license));
                        } else if key == "closed_source" {
                            return Some("This engine uses assets from the closed source release. Are you sure you want to continue?".to_string());
                        }
                    }
                }
            }
        }

        None
    }

    pub fn find_download_config_by_name(&self, name: &str) -> Option<DownloadConfig> {
        if let Some(download_config) = &self.download_config {
            return download_config
                .iter()
                .find(|x| x.download_name == name)
                .cloned();
        }

        None
    }
}
