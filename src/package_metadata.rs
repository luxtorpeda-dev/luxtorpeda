use log::{error, info};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::io::Error;
use std::path::{Path, PathBuf};
use url::Url;

use crate::config;
use crate::package;

const PACKAGE_METADATA_FILENAME: &str = "packagessniper_v2";

#[derive(Default, Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct PackageMetadata {
    pub games: Vec<Game>,
    pub engines: Vec<Engine>,
    pub default_engine: Game,
    pub notice_translation: Vec<NoticeTranslation>,
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
    pub setup: Option<Setup>,
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
    pub bchunk: Option<SetupBChunk>,
    pub iso_extract: Option<SetupIsoExtract>,
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
pub struct SetupBChunk {
    pub bin_file: String,
    pub cue_file: String,
    pub base_name: String,
    pub generate_cue_file: Option<SetupBChunkGenerateCueFile>,
}

#[derive(Default, Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct SetupBChunkGenerateCueFile {
    pub original: String,
    pub first_lines: usize,
}

#[derive(Default, Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct SetupIsoExtract {
    pub file_path: Option<String>,
    pub recursive_start_path: Option<String>,
    pub extract_prefix: Option<String>,
    pub extract_to_prefix: Option<String>,
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

#[derive(Default, Deserialize, Serialize, Debug, Clone)]
#[serde(default)]
pub struct NoticeTranslation {
    pub key: String,
    pub value: String,
}

impl PackageMetadata {
    pub fn from_packages_file() -> PackageMetadata {
        let packages_json_file = PackageMetadata::path_to_packages_file();
        if packages_json_file.exists() {
            info!("packages_json_file exists, reading");
            match fs::read_to_string(packages_json_file) {
                Ok(s) => match serde_json::from_str::<PackageMetadata>(&s) {
                    Ok(metadata) => {
                        let config = config::Config::from_config_file();
                        if let Some(remote_packages) = &config.additional_remote_packages {
                            PackageMetadata::from_remote_packages_cache(metadata, remote_packages)
                        } else {
                            metadata
                        }
                    }
                    Err(err) => {
                        error!("error parsing packages_json_file: {:?}", err);
                        Default::default()
                    }
                },
                Err(err) => {
                    error!("error reading packages_json_file: {:?}", err);
                    Default::default()
                }
            }
        } else {
            info!("packages_json_file does not exist");
            Default::default()
        }
    }

    pub fn from_remote_packages_cache(
        mut metadata: PackageMetadata,
        remote_packages: &[String],
    ) -> PackageMetadata {
        for url_str in remote_packages {
            info!(
                "from_remote_packages_cache. loading from cache for {}",
                url_str
            );

            let parsed_url = match Url::parse(url_str) {
                Ok(u) => u,
                Err(err) => {
                    error!("from_remote_packages_cache. url parse err: {:?}", err);
                    return metadata;
                }
            };

            let filename = match parsed_url.path_segments() {
                Some(mut segments) => match segments.next_back() {
                    Some(segment) => segment,
                    None => {
                        error!("from_remote_packages_cache. url last not found");
                        return metadata;
                    }
                },
                None => {
                    error!("from_remote_packages_cache. url path_segments not found");
                    return metadata;
                }
            };

            let local_packages_path =
                PackageMetadata::path_to_packages_file().with_file_name(filename);

            match fs::read_to_string(local_packages_path) {
                Ok(s) => match serde_json::from_str::<PackageMetadata>(&s) {
                    Ok(mut cached_metadata) => {
                        info!("merging cached remote package of {}", filename);
                        metadata.games.append(&mut cached_metadata.games);
                        metadata.engines.append(&mut cached_metadata.engines);
                    }
                    Err(err) => {
                        error!("error parsing from_remote_packages_cache: {:?}", err);
                        return metadata;
                    }
                },
                Err(err) => {
                    error!("error reading from_remote_packages_cache: {:?}", err);
                    return metadata;
                }
            }
        }

        metadata
    }

    pub fn find_game_by_app_id(&self, app_id: &str) -> Option<Game> {
        self.games.iter().find(|x| x.app_id == app_id).cloned()
    }

    pub fn find_engine_by_name(&self, name: &str) -> Option<Engine> {
        self.engines.iter().find(|x| x.engine_name == name).cloned()
    }

    pub fn find_notice_translation_by_key(&self, key: &str) -> Option<NoticeTranslation> {
        self.notice_translation
            .iter()
            .find(|x| x.key == key)
            .cloned()
    }

    pub fn convert_notice_to_str(&self, notice_item: &Notice) -> String {
        let mut notice = String::new();

        if let Some(label) = &notice_item.label {
            notice = label.to_string();
        } else if let Some(value) = &notice_item.value {
            if let Some(notice_translation) = self.find_notice_translation_by_key(value) {
                notice = notice_translation.value;
            } else {
                notice = value.to_string();
            }
        } else if let Some(key) = &notice_item.key {
            if let Some(notice_translation) = self.find_notice_translation_by_key(key) {
                notice = notice_translation.value;
            } else {
                notice = key.to_string();
            }
        }

        notice
    }

    pub fn update_packages_json() -> io::Result<()> {
        let config = config::Config::from_config_file();
        if !config.should_do_update {
            return PackageMetadata::download_additional_remote_packages(&config);
        }

        let packages_json_file = PackageMetadata::path_to_packages_file();
        let mut should_download = true;
        let mut remote_hash_str: String = String::new();

        let remote_path = PACKAGE_METADATA_FILENAME;

        let remote_hash_url = std::format!("{0}/{1}.hash256", config.host_url, remote_path);
        match PackageMetadata::get_remote_packages_hash(&remote_hash_url) {
            Some(tmp_hash_str) => {
                remote_hash_str = tmp_hash_str;
            }
            None => {
                info!("update_packages_json in get_remote_packages_hash call. received none");
                should_download = false;
            }
        }

        if should_download {
            if !Path::new(&packages_json_file).exists() {
                should_download = true;
                info!(
                    "update_packages_json. {:?} does not exist",
                    packages_json_file
                );
            } else {
                let hash_str = package::generate_hash_from_file_path(&packages_json_file)?;
                info!("update_packages_json. found hash: {}", hash_str);

                info!(
                    "update_packages_json. found hash and remote hash: {0} {1}",
                    hash_str, remote_hash_str
                );
                if hash_str != remote_hash_str {
                    info!("update_packages_json. hash does not match. downloading");
                    should_download = true;
                } else {
                    should_download = false;
                }
            }
        }

        if should_download {
            info!("update_packages_json. downloading new {}.json", remote_path);

            let remote_packages_url = std::format!("{0}/{1}.json", config.host_url, remote_path);
            let mut download_complete = false;
            let local_packages_temp_path = PackageMetadata::path_to_packages_file()
                .with_file_name(std::format!("{}-temp.json", remote_path));

            match reqwest::blocking::get(remote_packages_url) {
                Ok(mut response) => {
                    let mut dest = fs::File::create(&local_packages_temp_path)?;
                    io::copy(&mut response, &mut dest)?;
                    download_complete = true;
                }
                Err(err) => {
                    error!("update_packages_json. download err: {:?}", err);
                }
            }

            if download_complete {
                let new_hash_str =
                    package::generate_hash_from_file_path(&local_packages_temp_path)?;
                if new_hash_str == remote_hash_str {
                    info!("update_packages_json. new downloaded hash matches");
                    fs::rename(local_packages_temp_path, packages_json_file)?;
                } else {
                    info!("update_packages_json. new downloaded hash does not match");
                    fs::remove_file(local_packages_temp_path)?;
                }
            }
        }

        PackageMetadata::download_additional_remote_packages(&config)
    }

    pub fn download_additional_remote_packages(config: &config::Config) -> io::Result<()> {
        if let Some(additional_remote_packages) = &config.additional_remote_packages {
            for url_str in additional_remote_packages {
                info!(
                    "download_additional_remote_packages. downloading from {}",
                    url_str
                );

                let parsed_url = match Url::parse(url_str) {
                    Ok(u) => u,
                    Err(err) => {
                        let error_str = format!(
                            "download_additional_remote_packages. url parse err: {:?}",
                            err
                        );
                        error!("{}", error_str);
                        return Err(Error::other(error_str));
                    }
                };

                let filename = match parsed_url.path_segments() {
                    Some(mut segments) => match segments.next_back() {
                        Some(segment) => segment,
                        None => {
                            let error_str =
                                "download_additional_remote_packages. url last not found";
                            error!("{}", error_str);
                            return Err(Error::other(error_str));
                        }
                    },
                    None => {
                        let error_str =
                            "download_additional_remote_packages. url path_segments not found";
                        error!("{}", error_str);
                        return Err(Error::other(error_str));
                    }
                };

                let local_packages_path =
                    PackageMetadata::path_to_packages_file().with_file_name(filename);

                match reqwest::blocking::get(url_str) {
                    Ok(mut response) => {
                        let mut dest = fs::File::create(&local_packages_path)?;
                        io::copy(&mut response, &mut dest)?;
                        info!(
                            "download_additional_remote_packages {} is saved to {:?}",
                            url_str, local_packages_path
                        );
                    }
                    Err(err) => {
                        let error_str = format!(
                            "download_additional_remote_packages. download err: {:?}",
                            err
                        );
                        error!("{}", error_str);
                        return Err(Error::other(error_str));
                    }
                }
            }
        } else {
            info!("download_additional_remote_packages, no remote packages list given");
        }

        Ok(())
    }

    pub fn get_remote_packages_hash(remote_hash_url: &str) -> Option<String> {
        let remote_hash_response = match reqwest::blocking::get(remote_hash_url) {
            Ok(s) => s,
            Err(err) => {
                error!("get_remote_packages_hash error in get: {:?}", err);
                return None;
            }
        };

        let remote_hash_str = match remote_hash_response.text() {
            Ok(s) => s,
            Err(err) => {
                error!("get_remote_packages_hash error in text: {:?}", err);
                return None;
            }
        };

        Some(remote_hash_str)
    }

    fn path_to_packages_file() -> PathBuf {
        let xdg_dirs = xdg::BaseDirectories::with_prefix("luxtorpeda");
        let folder_path = xdg_dirs.get_cache_home().unwrap();
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
                    PackageMetadata::find_engine_by_name(&package_metadata, engine_name)
                {
                    if let Some(engine_notices) = engine.notices {
                        for entry in engine_notices {
                            choice_info
                                .notices
                                .push(PackageMetadata::convert_notice_to_str(
                                    &package_metadata,
                                    &entry,
                                ));
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
                            .push(PackageMetadata::convert_notice_to_str(
                                &package_metadata,
                                entry,
                            ));
                    }
                }

                simple_choices.push(choice_info);
            }
        }

        simple_choices
    }

    pub fn find_license_dialog_message(&self) -> Option<String> {
        let package_metadata = PackageMetadata::from_packages_file();
        if let Some(engine) =
            PackageMetadata::find_engine_by_name(&package_metadata, &self.engine_name)
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

    pub fn update_from_choice(&mut self, engine_choice: &EngineChoice) {
        if let Some(engine_name) = &engine_choice.engine_name {
            self.engine_name = engine_name.to_string();
        }

        if engine_choice.command.is_some() {
            self.command = engine_choice.command.clone();
        }

        if !engine_choice.command_args.is_empty() {
            self.command_args = engine_choice.command_args.clone();
        }

        if engine_choice.download_config.is_some() {
            self.download_config = engine_choice.download_config.clone();
        }

        if engine_choice.commands.is_some() {
            self.commands = engine_choice.commands.clone();
        }

        if engine_choice.setup.is_some() {
            self.setup = engine_choice.setup.clone();
        }
    }
}
