// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

mod api;

use std::{fmt, fs, path::Path};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use super::Result;
pub(crate) use api::{handle_gen_config, handle_save_config, handle_show_config};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(default = "default_mountsource")]
    pub mountsource: String,
    pub partitions: Vec<String>,
    #[serde(default)]
    pub extra_mount: Vec<String>,
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
            extra_mount: Vec::new(),
            umount: false,
        }
    }
}

impl Config {
    pub fn extra_mount_partitions(&self, root: &Path) -> Vec<String> {
        let mut partitions = Vec::new();
        for name in &self.extra_mount {
            if name.is_empty()
                || matches!(name.as_str(), "." | ".." | "system")
                || name.contains(['/', '\0'])
            {
                log::warn!("invalid extra_mount partition: {name:?}");
                continue;
            }
            if partitions.contains(name) {
                continue;
            }
            if !root.join(name).is_dir() {
                log::warn!("extra_mount {name}: target is not an existing directory");
                continue;
            }
            match fs::symlink_metadata(root.join("system").join(name)) {
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                    partitions.push(name.clone());
                }
                Ok(_) => log::warn!("extra_mount {name}: /system entry already exists"),
                Err(err) => log::warn!("extra_mount {name}: cannot check /system entry: {err}"),
            }
        }
        partitions
    }

    const fn umount_enabled(&self) -> bool {
        self.umount
    }

    const fn set_umount_enabled(&mut self, enabled: bool) {
        self.umount = enabled;
    }

    pub fn load<P>(path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        let content = fs::read_to_string(path).context("failed to read config file")?;

        let config: Self = toml::from_str(&content).unwrap_or_else(|e| {
            log::error!("Failed to deserialize config to toml: {e}");
            Self::default()
        });

        Ok(config)
    }

    pub fn load_or_default<P>(path: P) -> Self
    where
        P: AsRef<Path>,
    {
        match Self::load(path) {
            Ok(config) => config,
            Err(err) => {
                log::warn!("Failed to load config, using default: {err}");
                Self::default()
            }
        }
    }

    fn save<P>(&self, path: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let content = toml::to_string_pretty(self).context("failed to serialize config to toml")?;

        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent).context("failed to create config directory")?;
        }

        fs::write(path, content).context("failed to write config file")?;
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../tests/unit/config.rs"]
mod tests;
