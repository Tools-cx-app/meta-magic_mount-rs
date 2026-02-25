// Copyright 2025 Magic Mount-rs Authors
// SPDX-License-Identifier: GPL-3.0-or-later

use std::{fmt, fs};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::defs::CONFIG_FILE;

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
    pub fn load() -> Result<Self> {
        let content = fs::read_to_string(CONFIG_FILE).context("failed to read config file")?;

        let config: Self = toml::from_str(&content).map_or_else(
            |e| {
                log::error!("Failed to deserialize config to toml: {e}");
                Config::default()
            },
            |c| c,
        );

        Ok(config)
    }
}
