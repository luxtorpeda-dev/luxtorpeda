use crate::parsers::appinfo_vdf_parser::open_appinfo_vdf;
use serde_json::{Map, Value};
use std::fs;
use std::path::{Path, PathBuf};
use vdf_serde_format::from_str;
trait JsonExt {
    fn str_at(&self, key: &str) -> Option<&str>;
    fn u64_at(&self, key: &str) -> Option<u64>;
}

impl JsonExt for serde_json::Value {
    fn str_at(&self, key: &str) -> Option<&str> {
        self.get(key)?.as_str()
    }
    fn u64_at(&self, key: &str) -> Option<u64> {
        self.get(key)?.as_u64()
    }
}

fn get_app_info<'a>(appinfo_json: &'a Map<String, Value>, target_appid: u64) -> Option<&'a Value> {
    appinfo_json
        .get("entries")?
        .as_array()?
        .iter()
        .find(|entry| entry.u64_at("appid") == Some(target_appid))
}

fn compat_tools<'a>(manifest: &'a Value) -> impl Iterator<Item = (&'a str, &'a Value)> {
    manifest["extended"]
        .get("compat_tools")
        .and_then(|v| v.as_object())
        .into_iter()
        .flat_map(|map| map.iter())
        .filter(|(_, tool)| tool.str_at("from_oslist") == Some("windows"))
        .map(|(name, tool)| (name.as_str(), tool))
}

pub struct Tool {
    pub alias: String,
    pub display_name: String,
    pub commandline: String,
}

fn get_commandline(path: &impl AsRef<Path>) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let toolpath = path.as_ref().join("toolmanifest.vdf");
    let vdf = fs::read_to_string(&toolpath)?;
    let root: serde_json::Value = from_str(&vdf)?;

    if let Some(cmd) = root["manifest"]["commandline_waitforexitandrun"].as_str() {
        Ok(Some(cmd.replace(" waitforexitandrun", "")))
    } else if let Some(cmd) = root["manifest"]["commandline"].as_str() {
        Ok(Some(cmd.replace(" %verb%", "")))
    } else {
        Ok(None)
    }
}

pub fn find_tool<'a>(tools: &'a [Tool], alias: &str) -> Option<&'a Tool> {
    tools.iter().find(|t| t.alias == alias)
}

pub fn list_proton_tools(steam_path: &str) -> Result<Vec<Tool>, Box<dyn std::error::Error>> {
    let path = PathBuf::from(steam_path).join("appcache/appinfo.vdf");
    let appinfo_json = open_appinfo_vdf(&path);

    let manifests = get_app_info(&appinfo_json, 891390).unwrap();

    let mut tools = Vec::new();

    for (internal, tool) in compat_tools(manifests) {
        let Some(appid) = tool.u64_at("appid") else {
            continue;
        };
        let Some(app) = get_app_info(&appinfo_json, appid) else {
            continue;
        };
        let Some(path) = app["config"].str_at("installdir") else {
            continue;
        };

        let proton_path = PathBuf::from(steam_path)
            .join("steamapps/common")
            .join(path);

        if !proton_path.exists() {
            continue;
        }

        if let Some(display) = tool.str_at("display_name") {
            if let Some(cmd) = get_commandline(&proton_path)? {
                let finish_cmdline = format!("{}{}", proton_path.display().to_string(), cmd);
                tools.push(Tool {
                    alias: internal.to_string(),
                    display_name: display.to_string(),
                    commandline: finish_cmdline,
                });
            }
        }
    }

    Ok(tools)
}
