use new_vdf_parser::appinfo_vdf_parser::open_appinfo_vdf;
use keyvalues_serde::from_str_with_key;
use serde_json::{Map, Value};
use std::fs;
use std::path::{Path, PathBuf};
extern crate steamlocate;
use steamlocate::SteamDir;

const EXCLUDED_TOOLS: [&'static str; 7] = [
    "legacyruntime",
    "boxtron",
    "roberta",
    "luxtorpeda",
    "luxtorpeda-dev",
    "steam-play-none",
    "boson",
];

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

#[derive(Clone)]
pub struct Tool {
    pub alias: String,        // example: "proton_9"
    pub display_name: String, // example: "Proton 9.0-4"
    pub commandline: String, // example: "/home/mv/.steam/debian-installation/steamapps/common/Proton 9.0 (Beta)/proton"
}

fn get_commandline(path: &impl AsRef<Path>) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let toolpath = path.as_ref().join("toolmanifest.vdf");
    let vdf = fs::read_to_string(&toolpath)?;
    let (root, _key) = from_str_with_key::<serde_json::Map<String, serde_json::Value>>(&vdf)?;

    if let Some(cmd) = root
        .get("commandline_waitforexitandrun")
        .and_then(|v| v.as_str())
    {
        Ok(Some(cmd.replace(" waitforexitandrun", "")))
    } else if let Some(cmd) = root.get("commandline").and_then(|v| v.as_str()) {
        Ok(Some(cmd.replace(" %verb%", "")))
    } else {
        Ok(None)
    }
}

pub fn find_tool<'a>(tools: &'a [Tool], alias: &str) -> Option<&'a Tool> {
    tools.iter().find(|t| t.alias == alias)
}

pub fn find_tool_by_name<'a>(tools: &'a [Tool], display_name: &str) -> Option<&'a Tool> {
    tools.iter().find(|t| t.display_name == display_name)
}

// List tools from .steam/steam/compatibilitytools.d
pub fn list_compatibilitytoolsd(steam_path: &str) -> Result<Vec<Tool>, Box<dyn std::error::Error>> {
    let mut tools = Vec::new();

    let compattoolsd = PathBuf::from(steam_path).join("compatibilitytools.d");
    let paths = fs::read_dir(&compattoolsd)?;

    for path in paths {
        let tool_dir = path?.path();

        // Exclude tools such as "Luxtorpeda" or "Roberta" as we're only going to want Proton or similar.
        if let Some(name) = tool_dir.file_name().and_then(|n| n.to_str()) {
            if EXCLUDED_TOOLS.contains(&name.to_lowercase().as_str()) {
                continue;
            }
        }

        let compatvdfpath = tool_dir.join("compatibilitytool.vdf");
        if !compatvdfpath.exists() {
            continue;
        }

        let vdf = fs::read_to_string(&compatvdfpath)?;
        let (root, _key) = from_str_with_key::<serde_json::Map<String, serde_json::Value>>(&vdf)?;

        // Look inside compat_tools
        if let Some(compat_tools) = root.get("compat_tools").and_then(|v| v.as_object()) {
            for (alias, entry) in compat_tools {
                if let Some(entry) = entry.as_object() {
                    // Filter out non-Windows tools
                    if entry
                        .get("from_oslist")
                        .and_then(|v| v.as_str())
                        .map(|s| s != "windows")
                        .unwrap_or(true)
                    {
                        continue;
                    }

                    // Get display name (what is shown in Steam GUI)
                    let display_name = entry
                        .get("display_name")
                        .and_then(|v| v.as_str())
                        .unwrap_or(alias);

                    // Get path to the Proton tool
                    let install_path = entry
                        .get("install_path")
                        .and_then(|v| v.as_str())
                        .unwrap_or(".");

                    let resolved_path = tool_dir.join(install_path);

                    let Some(commandline) = get_commandline(&resolved_path)? else {
                        continue;
                    };

                    let finish_cmdline =
                        format!("{}{}", resolved_path.display().to_string(), commandline);

                    tools.push(Tool {
                        alias: alias.clone(),
                        display_name: display_name.to_string(),
                        commandline: finish_cmdline,
                    });
                }
            }
        }
    }

    Ok(tools)
}

pub fn list_valve_proton_tools(steam_path: &str) -> Result<Vec<Tool>, Box<dyn std::error::Error>> {
    let path = PathBuf::from(steam_path).join("appcache/appinfo.vdf");
    let appinfo_json = open_appinfo_vdf(&path, Some(false));

    let manifests = get_app_info(&appinfo_json, 891390).unwrap();

    let mut tools = Vec::new();

    for (internal, tool) in compat_tools(manifests) {
        let Some(appid) = tool.u64_at("appid") else {
            continue;
        };

        let Some(mut steam_dir) = SteamDir::locate() else {
            continue;
        };

        let Some(app) = steam_dir.app(&appid.try_into().unwrap()) else {
            continue;
        };

        let proton_path = app.path.clone();

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

pub fn list_proton_tools(steam_path: &str) -> Result<Vec<Tool>, Box<dyn std::error::Error>> {
    let mut tools = Vec::new();

    let official_tools = list_valve_proton_tools(steam_path)?;
    let unofficial_tools = list_compatibilitytoolsd(steam_path)?;

    tools.extend(official_tools);
    tools.extend(unofficial_tools);

    Ok(tools)
}
