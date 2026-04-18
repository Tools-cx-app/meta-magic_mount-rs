// Copyright (C) 2026 Tools-cx-app <localhost.hutao@gmail.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::{fmt, fs};

use anyhow::{Context, Result, anyhow, bail};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    defs::{CONFIG_FILE, IGNORE_LIST_PATH, MODULE_PATH},
    magic_mount::node::IGNORE_LIST,
};

#[derive(Debug, Serialize)]
pub struct ApiConfig {
    pub moduledir: String,
    pub mountsource: String,
    pub partitions: Vec<String>,
    pub umount: bool,
    pub disable_umount: bool,
    #[serde(rename = "ignoreList")]
    pub ignore_list: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct ApiConfigPayload {
    pub mountsource: Option<String>,
    pub partitions: Option<Vec<String>>,
    pub umount: Option<bool>,
    pub disable_umount: Option<bool>,
    #[serde(rename = "ignoreList", alias = "ignore_list")]
    pub ignore_list: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(default = "default_mountsource")]
    pub mountsource: String,
    pub partitions: Vec<String>,
    #[cfg(any(target_os = "linux", target_os = "android"))]
    pub umount: bool,
}

fn default_mountsource() -> String {
    String::from("KSU")
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let toml = toml::to_string_pretty(self)
            .context("Failed to serialize config to toml")
            .unwrap();
        write!(f, "{toml}")
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            mountsource: default_mountsource(),
            partitions: Vec::new(),
            #[cfg(any(target_os = "linux", target_os = "android"))]
            umount: false,
        }
    }
}

impl Config {
    #[cfg(any(target_os = "linux", target_os = "android"))]
    const fn umount_enabled(&self) -> bool {
        self.umount
    }

    #[cfg(not(any(target_os = "linux", target_os = "android")))]
    const fn umount_enabled(&self) -> bool {
        false
    }

    #[cfg(any(target_os = "linux", target_os = "android"))]
    const fn set_umount_enabled(&mut self, enabled: bool) {
        self.umount = enabled;
    }

    #[cfg(not(any(target_os = "linux", target_os = "android")))]
    const fn set_umount_enabled(&mut self, _enabled: bool) {}

    pub fn load() -> Result<Self> {
        let content = fs::read_to_string(CONFIG_FILE).context("failed to read config file")?;

        let config: Self = toml::from_str(&content).unwrap_or_else(|e| {
            log::error!("Failed to deserialize config to toml: {e}");
            Self::default()
        });

        Ok(config)
    }

    pub fn load_or_default() -> Self {
        match Self::load() {
            Ok(config) => config,
            Err(err) => {
                log::warn!("Failed to load config, using default: {err}");
                Self::default()
            }
        }
    }

    fn save(&self) -> Result<()> {
        let content = toml::to_string_pretty(self).context("failed to serialize config to toml")?;

        if let Some(parent) = std::path::Path::new(CONFIG_FILE).parent() {
            fs::create_dir_all(parent).context("failed to create config directory")?;
        }

        fs::write(CONFIG_FILE, content).context("failed to write config file")?;
        Ok(())
    }

    fn write_ignore_list(ignore_list: &[String]) -> Result<()> {
        if let Some(parent) = std::path::Path::new(IGNORE_LIST_PATH).parent() {
            fs::create_dir_all(parent).context("failed to create ignore list directory")?;
        }

        let mut content = ignore_list.join("\n");
        if !content.is_empty() {
            content.push('\n');
        }

        fs::write(IGNORE_LIST_PATH, content).context("failed to write ignore list")?;
        Ok(())
    }

    fn into_api(self, ignore_list: Vec<String>) -> ApiConfig {
        let umount_enabled = self.umount_enabled();

        ApiConfig {
            moduledir: MODULE_PATH.trim_end_matches('/').to_string(),
            mountsource: self.mountsource,
            partitions: self.partitions,
            umount: umount_enabled,
            disable_umount: !umount_enabled,
            ignore_list,
        }
    }

    fn apply_api_payload(&mut self, payload: ApiConfigPayload) {
        if let Some(mountsource) = payload.mountsource {
            self.mountsource = mountsource;
        }

        if let Some(partitions) = payload.partitions {
            self.partitions = partitions;
        }

        if let Some(umount) = payload.umount {
            self.set_umount_enabled(umount);
        } else if let Some(disable_umount) = payload.disable_umount {
            self.set_umount_enabled(!disable_umount);
        }
    }
}

pub fn decode_hex(input: &str) -> Result<Vec<u8>> {
    if !input.len().is_multiple_of(2) {
        bail!("hex payload must contain an even number of characters");
    }

    let mut bytes = Vec::with_capacity(input.len() / 2);
    for chunk in input.as_bytes().chunks_exact(2) {
        let hex = std::str::from_utf8(chunk).context("hex payload is not valid utf-8")?;
        let byte = u8::from_str_radix(hex, 16)
            .with_context(|| format!("invalid hex byte '{hex}' in payload"))?;
        bytes.push(byte);
    }

    Ok(bytes)
}

pub fn parse_payload_arg(args: &[String]) -> Result<&str> {
    let payload = args
        .windows(2)
        .find_map(|window| (window[0] == "--payload").then_some(window[1].as_str()))
        .ok_or_else(|| anyhow!("missing required --payload argument"))?;

    Ok(payload)
}

pub fn handle_show_config() -> Result<()> {
    let config = Config::load_or_default();
    let ignore_list: Vec<_> = IGNORE_LIST
        .get()
        .unwrap()
        .clone()
        .unwrap_or_default()
        .iter()
        .cloned()
        .collect();
    println!("{}", serde_json::to_string(&config.into_api(ignore_list))?);
    Ok(())
}

pub fn handle_save_config(args: &[String]) -> Result<()> {
    let payload_hex = parse_payload_arg(args)?;
    let payload_json = String::from_utf8(decode_hex(payload_hex)?)
        .context("decoded payload is not valid utf-8")?;
    let payload: ApiConfigPayload =
        serde_json::from_str(&payload_json).context("failed to parse config payload json")?;

    let ignore_list = payload.ignore_list.clone();
    let mut config = Config::load_or_default();
    config.apply_api_payload(payload);
    config.save()?;
    if let Some(ignore_list) = ignore_list {
        Config::write_ignore_list(&ignore_list)?;
    }

    println!("{}", json!({ "ok": true }));
    Ok(())
}

pub fn handle_gen_config() -> Result<()> {
    let config = Config::default();
    config.save()?;
    Config::write_ignore_list(&[])?;
    println!("{}", json!({ "ok": true }));
    Ok(())
}
