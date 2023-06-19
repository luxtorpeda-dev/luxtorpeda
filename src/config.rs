use log::{error, info};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::package;

#[derive(Deserialize, Serialize, Debug)]
#[serde(default)]
pub struct Config {
    pub host_url: String,
    pub should_do_update: bool,
    pub disable_default_confirm: bool,
    pub enable_steam_cloud: bool,
    pub hash_check_install: bool,
    pub close_client_on_launch: bool,
}

impl Default for Config {
    fn default() -> Config {
        Config {
            host_url: "https://luxtorpeda-dev.github.io".to_string(),
            should_do_update: true,
            disable_default_confirm: false,
            enable_steam_cloud: false,
            hash_check_install: true,
            close_client_on_launch: false,
        }
    }
}

impl Config {
    pub fn from_config_file() -> Config {
        let config_file_path = Config::config_file_path();
        if config_file_path.exists() {
            info!("config_file_path exists, reading");
            match fs::read_to_string(config_file_path) {
                Ok(s) => match serde_json::from_str::<Config>(&s) {
                    Ok(config) => config,
                    Err(err) => {
                        error!("error parsing config_file: {:?}", err);
                        Config::default_config_and_save()
                    }
                },
                Err(err) => {
                    error!("error reading config_file: {:?}", err);
                    Config::default_config_and_save()
                }
            }
        } else {
            info!("config_file_path does not exist, using default");
            Config::default_config_and_save()
        }
    }

    fn config_file_path() -> PathBuf {
        let config_path = package::path_to_config();
        config_path.join("config.json")
    }

    fn default_config_and_save() -> Config {
        let default_config: Config = Default::default();
        let config_file_path = Config::config_file_path();
        info!("writing config_file to {:?}", config_file_path);

        serde_json::to_string_pretty(&default_config)
            .ok()
            .and_then(|config_json| std::fs::write(config_file_path, config_json).ok());

        default_config
    }
}
