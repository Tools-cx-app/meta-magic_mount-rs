// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: Apache-2.0

use std::{fmt, fs, path::Path};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::{
    defs,
    errors::{Error, Result},
    parser::{COMMAND_LIST, Command, parser_custom},
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
                    Command::Ignore { source } => ignore_list.push(source),
                    Command::Mount { source, target } => {
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

pub fn decode_hex(input: &str) -> Result<Vec<u8>> {
    if !input.len().is_multiple_of(2) {
        return Err(Error::PayloadContain);
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
        .ok_or_else(|| Error::MissingArgment)?;

    Ok(payload)
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
                        Command::Ignore { source } => ignore_list.push(source),
                        Command::Mount { source, target } => {
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

pub fn handle_save_config(args: &[String]) -> Result<()> {
    let payload_hex = parse_payload_arg(args)?;
    let payload_json = String::from_utf8(decode_hex(payload_hex)?)
        .context("decoded payload is not valid utf-8")?;
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
mod tests {
    use super::*;

    #[test]
    fn test_decode_hex_valid_cases() {
        let input_lowercase = "74657374";
        let input_uppercase = "54455354";
        let input_extremes = "00ff00ff";

        assert_eq!(decode_hex(input_lowercase).unwrap(), b"test".to_vec());
        assert_eq!(decode_hex(input_uppercase).unwrap(), b"TEST".to_vec());
        assert_eq!(decode_hex("").unwrap(), Vec::<u8>::new());
        assert_eq!(
            decode_hex(input_extremes).unwrap(),
            vec![0x00, 0xFF, 0x00, 0xFF]
        );
    }

    #[test]
    fn test_decode_hex_invalid_length_error() {
        let input_odd_1 = "a";
        let input_odd_5 = "12345";

        assert!(matches!(
            decode_hex(input_odd_1),
            Err(Error::PayloadContain)
        ));
        assert!(matches!(
            decode_hex(input_odd_5),
            Err(Error::PayloadContain)
        ));
    }

    #[test]
    fn test_parse_payload_arg_success() {
        let args = vec![
            "magic_mount".to_string(),
            "--payload".to_string(),
            "7b7d".to_string(),
        ];
        assert_eq!(parse_payload_arg(&args).unwrap(), "7b7d");
    }

    #[test]
    fn test_parse_payload_arg_missing() {
        let args = vec!["magic_mount".to_string(), "--wrong-flag".to_string()];
        assert!(parse_payload_arg(&args).is_err());
    }

    #[test]
    fn test_format_custom_path() {
        assert_eq!(
            Config::format_custom_path("/system/bin/sh"),
            "/system/bin/sh"
        );

        assert_eq!(
            Config::format_custom_path("/data/local/my module name"),
            "\"/data/local/my module name\""
        );
    }

    #[test]
    fn test_apply_api_payload() {
        let mut config = Config::default();

        let payload = ApiConfigPayload {
            mountsource: Some("APatch".to_string()),
            partitions: Some(vec!["system".to_string(), "product".to_string()]),
            umount: Some(true),
            disable_umount: None,
            ignore_list: None,
            custom_mounts: None,
        };

        config.apply_api_payload(payload);
        assert_eq!(config.mountsource, "APatch");
        assert_eq!(
            config.partitions,
            vec!["system".to_string(), "product".to_string()]
        );
        assert!(config.umount);
    }

    #[test]
    fn test_apply_api_payload_disable_umount() {
        let mut config = Config {
            mountsource: "KSU".to_string(),
            partitions: vec![],
            umount: true,
        };

        let payload = ApiConfigPayload {
            mountsource: None,
            partitions: None,
            umount: None,
            disable_umount: Some(true),
            ignore_list: None,
            custom_mounts: None,
        };

        config.apply_api_payload(payload);
        assert!(!config.umount);
    }

    #[test]
    fn test_config_save_and_load_flow() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let config_file_path = tmp_dir.path().join("config.toml");

        let mut config = Config::default();
        config.mountsource = "Magisk".to_string();
        config.partitions = vec!["vendor".to_string()];
        config.umount = true;
        assert!(config.save(&config_file_path).is_ok());

        let loaded_config = Config::load(&config_file_path).unwrap();
        assert_eq!(loaded_config.mountsource, "Magisk");
        assert_eq!(loaded_config.partitions, vec!["vendor".to_string()]);
        assert!(loaded_config.umount);
    }

    #[test]
    fn test_load_or_default_fallback() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let non_existent = tmp_dir.path().join("missing.toml");

        let config = Config::load_or_default(&non_existent);
        assert_eq!(config.mountsource, "KSU");
        assert!(!config.umount);
    }

    #[test]
    fn test_write_custom_list_serialization() {
        let tmp_dir = tempfile::tempdir().unwrap();
        let list_file_path = tmp_dir.path().join("custom.list");

        let ignore_list = vec!["/system/app".to_string()];
        let custom_mounts = vec![ApiCustomMount {
            source: "/data/local/test app".to_string(),
            target: "/system/priv-app".to_string(),
        }];

        assert!(Config::write_custom_list(&list_file_path, &ignore_list, &custom_mounts).is_ok());

        let file_content = fs::read_to_string(&list_file_path).unwrap();
        let lines: Vec<&str> = file_content.lines().collect();

        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0], "ignore /system/app");
        assert_eq!(lines[1], "bind \"/data/local/test app\" /system/priv-app");
    }

    #[test]
    fn test_into_api_struct_mapping() {
        let config = Config {
            mountsource: "KSU".to_string(),
            partitions: vec!["system".to_string()],
            umount: false,
        };
        let ignore_list = vec!["/data/local/tmp".to_string()];
        let custom_mounts = vec![];

        let api_config = config.into_api(ignore_list, custom_mounts);
        assert_eq!(api_config.mountsource, "KSU");
        assert_eq!(api_config.partitions, vec!["system".to_string()]);
        assert!(!api_config.umount);
        assert!(api_config.disable_umount);
        assert_eq!(api_config.ignore_list[0], "/data/local/tmp");
    }
}
