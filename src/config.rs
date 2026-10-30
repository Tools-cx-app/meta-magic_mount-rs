// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use std::{fmt, fs, path::Path};

use anyhow::Context;
use hex::decode;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{defs, errors::Result};

#[derive(Debug, Serialize)]
pub struct ApiConfig {
    pub moduledir: String,
    pub mountsource: String,
    pub partitions: Vec<String>,
    pub extra_mount: Vec<String>,
    pub umount: bool,
    pub disable_umount: bool,
}

#[derive(Debug, Deserialize)]
pub struct ApiConfigPayload {
    pub mountsource: Option<String>,
    pub partitions: Option<Vec<String>>,
    pub extra_mount: Option<Vec<String>>,
    pub umount: Option<bool>,
    pub disable_umount: Option<bool>,
}

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

    fn into_api(self) -> ApiConfig {
        let umount_enabled = self.umount_enabled();

        ApiConfig {
            moduledir: defs::MODULE_PATH.trim_end_matches('/').to_string(),
            mountsource: self.mountsource,
            partitions: self.partitions,
            extra_mount: self.extra_mount,
            umount: umount_enabled,
            disable_umount: !umount_enabled,
        }
    }

    fn apply_api_payload(&mut self, payload: ApiConfigPayload) {
        if let Some(mountsource) = payload.mountsource {
            self.mountsource = mountsource;
        }

        if let Some(partitions) = payload.partitions {
            self.partitions = partitions;
        }

        if let Some(extra_mount) = payload.extra_mount {
            self.extra_mount = extra_mount;
        }

        if let Some(umount) = payload.umount {
            self.set_umount_enabled(umount);
        } else if let Some(disable_umount) = payload.disable_umount {
            self.set_umount_enabled(!disable_umount);
        }
    }
}

pub fn handle_show_config() -> Result<()> {
    let config = Config::load_or_default(defs::CONFIG_FILE);

    println!("{}", serde_json::to_string(&config.into_api())?);
    Ok(())
}

pub fn handle_save_config(payload_hex: &str) -> Result<()> {
    let payload_json =
        String::from_utf8(decode(payload_hex)?).context("decoded payload is not valid utf-8")?;
    let payload: ApiConfigPayload =
        serde_json::from_str(&payload_json).context("failed to parse config payload json")?;

    let mut config = Config::load_or_default(defs::CONFIG_FILE);
    config.apply_api_payload(payload);
    config.save(defs::CONFIG_FILE)?;

    println!("{}", json!({ "ok": true }));
    Ok(())
}

pub fn handle_gen_config() -> Result<()> {
    let config = Config::default();
    config.save(defs::CONFIG_FILE)?;
    println!("{}", json!({ "ok": true }));
    Ok(())
}
#[cfg(test)]
#[path = "../tests/unit/config.rs"]
mod tests;
