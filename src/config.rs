// Copyright 2025 Magic Mount-rs Authors
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{fmt, fs};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::defs::{CONFIG_FILE, IGNORE_LIST_PATH, MODULE_PATH};

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
    fn umount_enabled(&self) -> bool {
        self.umount
    }

    #[cfg(not(any(target_os = "linux", target_os = "android")))]
    fn umount_enabled(&self) -> bool {
        false
    }

    #[cfg(any(target_os = "linux", target_os = "android"))]
    fn set_umount_enabled(&mut self, enabled: bool) {
        self.umount = enabled;
    }

    #[cfg(not(any(target_os = "linux", target_os = "android")))]
    fn set_umount_enabled(&mut self, _enabled: bool) {}

    pub fn load() -> Result<Self> {
        let content = fs::read_to_string(CONFIG_FILE).context("failed to read config file")?;

        let config: Self = toml::from_str(&content).unwrap_or_else(|e| {
            log::error!("Failed to deserialize config to toml: {e}");
            Self::default()
        });

        Ok(config)
    }

    pub fn load_or_default() -> Result<Self> {
        match Self::load() {
            Ok(config) => Ok(config),
            Err(err) => {
                log::warn!("Failed to load config, using default: {err}");
                Ok(Self::default())
            }
        }
    }

    pub fn save(&self) -> Result<()> {
        let content = toml::to_string_pretty(self).context("failed to serialize config to toml")?;

        if let Some(parent) = std::path::Path::new(CONFIG_FILE).parent() {
            fs::create_dir_all(parent).context("failed to create config directory")?;
        }

        fs::write(CONFIG_FILE, content).context("failed to write config file")?;
        Ok(())
    }

    pub fn read_ignore_list() -> Result<Vec<String>> {
        let content = match fs::read_to_string(IGNORE_LIST_PATH) {
            Ok(content) => content,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(err) => return Err(err).context("failed to read ignore list"),
        };

        Ok(content
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(ToOwned::to_owned)
            .collect())
    }

    pub fn write_ignore_list(ignore_list: &[String]) -> Result<()> {
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

    pub fn into_api(self, ignore_list: Vec<String>) -> ApiConfig {
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

    pub fn apply_api_payload(&mut self, payload: ApiConfigPayload) {
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
