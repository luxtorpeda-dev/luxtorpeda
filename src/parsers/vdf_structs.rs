/*
 * This file is part of Steam-Art-Manager which is licensed under GNU Lesser General Public License v2.1
 * See file LICENSE or go to https://www.gnu.org/licenses/old-licenses/lgpl-2.1.en.html for full license details
 */

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[allow(non_snake_case)]
pub struct User {
    pub AccountName: String,
    pub PersonaName: String,
    pub RememberPassword: String,
    pub WantsOfflineMode: String,
    pub SkipOfflineModeWarning: String,
    pub AllowAutoLogin: String,
    pub MostRecent: String,
    pub TimeStamp: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[allow(non_snake_case)]
pub struct HKLMSteam {
    pub SteamPID: String,
    pub TempAppCmdLine: String,
    pub ReLaunchCmdLine: String,
    pub ClientLauncher: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[allow(non_snake_case)]
pub struct HKLMValve {
    pub Steam: HKLMSteam,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[allow(non_snake_case)]
pub struct HKLMSoftware {
    pub Valve: HKLMValve,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[allow(non_snake_case)]
pub struct HKLM {
    pub Software: HKLMSoftware,
}

// #[derive(Serialize, Deserialize, Debug, PartialEq)]
// #[allow(non_snake_case)]
// pub struct HKCUApp {
//   pub Updating: String,
//   pub installed: String,
//   pub Running: String,
//   pub name: String
// }

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[allow(non_snake_case)]
pub struct HKCUSteamGlobal {
    pub language: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[allow(non_snake_case)]
pub struct HKCUSteam {
    pub RunningAppID: String,
    pub steamglobal: HKCUSteamGlobal,
    pub language: String,
    pub Completed00BE: String,
    pub SourceModInstallPath: String,
    pub AutoLoginUser: String,
    pub Rate: String,
    pub AlreadyRetriedOfflineMode: String,
    pub apps: HashMap<String, HashMap<String, String>>,
    pub StartupMode: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[allow(non_snake_case)]
pub struct HKCUValve {
    pub Steam: HKCUSteam,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[allow(non_snake_case)]
pub struct HKCUSoftware {
    pub Valve: HKCUValve,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[allow(non_snake_case)]
pub struct HKCU {
    pub Software: HKCUSoftware,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[allow(non_snake_case)]
pub struct Registry {
    pub HKLM: HKLM,
    pub HKCU: HKCU,
}
