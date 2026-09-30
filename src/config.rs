// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use std::{fmt, fs, path::Path};

use anyhow::Context;
use hex::decode;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    defs,
    errors::Result,
    parser::{COMMAND_LIST, MountType, parser_custom},
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiCustomMount {
    pub source: String,
    pub target: String,
}

#[derive(Debug, Serialize)]
pub struct ApiConfig {
    pub moduledir: String,
    pub mountsource: String,
    pub partitions: Vec<String>,
    pub umount: bool,
    pub disable_umount: bool,
    #[serde(rename = "ignoreList")]
    pub ignore_list: Vec<String>,
    #[serde(rename = "customMounts")]
    pub custom_mounts: Vec<ApiCustomMount>,
}

#[derive(Debug, Deserialize)]
pub struct ApiConfigPayload {
    pub mountsource: Option<String>,
    pub partitions: Option<Vec<String>>,
    pub umount: Option<bool>,
    pub disable_umount: Option<bool>,
    #[serde(rename = "ignoreList", alias = "ignore_list")]
    pub ignore_list: Option<Vec<String>>,
    #[serde(rename = "customMounts", alias = "custom_mounts")]
    pub custom_mounts: Option<Vec<ApiCustomMount>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    #[serde(default = "default_mountsource")]
    pub mountsource: String,
    pub partitions: Vec<String>,
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
            umount: false,
        }
    }
}

impl Config {
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

    fn read_custom_lists<P>(path: P) -> (Vec<String>, Vec<ApiCustomMount>)
    where
        P: AsRef<Path>,
    {
        parser_custom(path).into_iter().fold(
            (Vec::new(), Vec::new()),
            |(mut ignore_list, mut custom_mounts), command| {
                match command {
                    MountType::Ignore { source } => ignore_list.push(source),
                    MountType::Mount { source, target } => {
                        custom_mounts.push(ApiCustomMount { source, target });
                    }
                }

                (ignore_list, custom_mounts)
            },
        )
    }

    fn format_custom_path(path: &str) -> String {
        if path.contains(char::is_whitespace) {
            format!("\"{path}\"")
        } else {
            path.to_string()
        }
    }

    fn write_custom_list<P>(
        path: P,
        ignore_list: &[String],
        custom_mounts: &[ApiCustomMount],
    ) -> Result<()>
    where
        P: AsRef<Path>,
    {
        if let Some(parent) = path.as_ref().parent() {
            fs::create_dir_all(parent).context("failed to create custom list directory")?;
        }

        let mut lines: Vec<String> = ignore_list
            .iter()
            .map(|source| format!("ignore {}", Self::format_custom_path(source)))
            .collect();
        lines.extend(custom_mounts.iter().map(|mount| {
            format!(
                "bind {} {}",
                Self::format_custom_path(&mount.source),
                Self::format_custom_path(&mount.target)
            )
        }));

        let mut content = lines.join("\n");
        if !content.is_empty() {
            content.push('\n');
        }

        fs::write(path, content).context("failed to write custom list")?;
        Ok(())
    }

    fn into_api(self, ignore_list: Vec<String>, custom_mounts: Vec<ApiCustomMount>) -> ApiConfig {
        let umount_enabled = self.umount_enabled();

        ApiConfig {
            moduledir: defs::MODULE_PATH.trim_end_matches('/').to_string(),
            mountsource: self.mountsource,
            partitions: self.partitions,
            umount: umount_enabled,
            disable_umount: !umount_enabled,
            ignore_list,
            custom_mounts,
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

pub fn handle_show_config() -> Result<()> {
    let config = Config::load_or_default(defs::CONFIG_FILE);
    let (ignore_list, custom_mounts) = COMMAND_LIST.get().map_or_else(
        || Config::read_custom_lists(defs::CUSTOM_LIST_PATH),
        |commands| {
            commands.iter().cloned().fold(
                (Vec::new(), Vec::new()),
                |(mut ignore_list, mut custom_mounts), command| {
                    match command {
                        MountType::Ignore { source } => ignore_list.push(source),
                        MountType::Mount { source, target } => {
                            custom_mounts.push(ApiCustomMount { source, target });
                        }
                    }

                    (ignore_list, custom_mounts)
                },
            )
        },
    );

    println!(
        "{}",
        serde_json::to_string(&config.into_api(ignore_list, custom_mounts))?
    );
    Ok(())
}

pub fn handle_save_config(payload_hex: &str) -> Result<()> {
    let payload_json =
        String::from_utf8(decode(payload_hex)?).context("decoded payload is not valid utf-8")?;
    let payload: ApiConfigPayload =
        serde_json::from_str(&payload_json).context("failed to parse config payload json")?;

    let ignore_list = payload.ignore_list.clone();
    let custom_mounts = payload.custom_mounts.clone();
    let mut config = Config::load_or_default(defs::CONFIG_FILE);
    config.apply_api_payload(payload);
    config.save(defs::CONFIG_FILE)?;
    if ignore_list.is_some() || custom_mounts.is_some() {
        let (current_ignore_list, current_custom_mounts) =
            Config::read_custom_lists(defs::CUSTOM_LIST_PATH);
        let ignore_list = ignore_list.unwrap_or(current_ignore_list);
        let custom_mounts = custom_mounts.unwrap_or(current_custom_mounts);

        Config::write_custom_list(defs::CUSTOM_LIST_PATH, &ignore_list, &custom_mounts)?;
    }

    println!("{}", json!({ "ok": true }));
    Ok(())
}

pub fn handle_gen_config() -> Result<()> {
    let config = Config::default();
    config.save(defs::CONFIG_FILE)?;
    Config::write_custom_list(defs::CUSTOM_LIST_PATH, &[], &[])?;
    println!("{}", json!({ "ok": true }));
    Ok(())
}
#[cfg(test)]
#[path = "../tests/unit/config.rs"]
mod tests;
