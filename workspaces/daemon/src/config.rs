// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use std::{
    io,
    path::{Component, Path, PathBuf},
    sync::Arc,
};

use anyhow::Context;
use api::{ApiConfig, CustomMount};
use rustc_hash::FxHashSet;
use serde::{Deserialize, Serialize};
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;

#[derive(Debug, Deserialize, Serialize)]
struct DiskConfig {
    #[serde(default = "default_mountsource")]
    mountsource: String,
    #[serde(default)]
    partitions: Vec<String>,
    #[serde(default)]
    umount: bool,
}

fn default_mountsource() -> String {
    "KSU".into()
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("{0}")]
    Invalid(String),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

#[derive(Clone)]
pub struct Store {
    config_path: PathBuf,
    custom_list_path: PathBuf,
    access: Arc<RwLock<()>>,
}

impl Store {
    pub fn new(config_path: impl Into<PathBuf>, custom_list_path: impl Into<PathBuf>) -> Self {
        Self {
            config_path: config_path.into(),
            custom_list_path: custom_list_path.into(),
            access: Arc::new(RwLock::new(())),
        }
    }

    pub async fn load(&self) -> anyhow::Result<ApiConfig> {
        let _guard = self.access.read().await;
        let disk: DiskConfig = toml::from_str(
            &tokio::fs::read_to_string(&self.config_path)
                .await
                .with_context(|| format!("failed to read {}", self.config_path.display()))?,
        )
        .context("failed to parse config")?;
        let (ignore_list, custom_mounts) = self.load_custom_list().await?;

        Ok(ApiConfig {
            mountsource: disk.mountsource,
            umount: disk.umount,
            partitions: disk.partitions,
            ignore_list,
            custom_mounts,
        })
    }

    pub async fn save(&self, mut config: ApiConfig) -> Result<(), ConfigError> {
        let _guard = self.access.write().await;
        normalize(&mut config)?;
        let disk = toml::to_string_pretty(&DiskConfig {
            mountsource: config.mountsource,
            partitions: config.partitions,
            umount: config.umount,
        })
        .context("failed to serialize config")?;
        let custom = format_custom_list(&config.ignore_list, &config.custom_mounts)?;
        let previous_config = read_optional(&self.config_path).await?;

        let config_temp = temporary_path(&self.config_path);
        let custom_temp = temporary_path(&self.custom_list_path);
        if let Err(error) = write_temporary(&config_temp, disk.as_bytes()).await {
            return Err(error.into());
        }
        if let Err(error) = write_temporary(&custom_temp, custom.as_bytes()).await {
            let _ = tokio::fs::remove_file(&config_temp).await;
            return Err(error.into());
        }

        if let Err(error) = tokio::fs::rename(&config_temp, &self.config_path).await {
            let _ = tokio::fs::remove_file(&config_temp).await;
            let _ = tokio::fs::remove_file(&custom_temp).await;
            return Err(anyhow::Error::new(error).into());
        }
        if let Err(error) = tokio::fs::rename(&custom_temp, &self.custom_list_path).await {
            let _ = tokio::fs::remove_file(&custom_temp).await;
            if let Err(rollback) = restore_optional(&self.config_path, previous_config).await {
                return Err(anyhow::anyhow!(
                    "failed to save custom rules: {error}; failed to restore config: {rollback}"
                )
                .into());
            }
            return Err(anyhow::Error::new(error).into());
        }
        Ok(())
    }

    async fn load_custom_list(&self) -> anyhow::Result<(Vec<String>, Vec<CustomMount>)> {
        enum Work {
            File(PathBuf),
            Line(String),
        }

        let mut ignore_list = Vec::new();
        let mut custom_mounts = Vec::new();
        let mut visited = FxHashSet::default();
        let mut work = vec![Work::File(self.custom_list_path.clone())];
        while let Some(item) = work.pop() {
            let Work::File(path) = item else {
                let Work::Line(line) = item else {
                    unreachable!();
                };
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                let tokens = tokenize(line);
                match tokens.as_slice() {
                    [command, source, ..] if command == "ignore" => {
                        let source = parse_path(source);
                        if !source.is_empty() {
                            ignore_list.push(source);
                        } else {
                            log::warn!("malformed command: {line}");
                        }
                    }
                    [command, source, target, ..] if command == "bind" => {
                        let source = parse_path(source);
                        let target = parse_path(target);
                        if !source.is_empty() && !target.is_empty() {
                            custom_mounts.push(CustomMount { source, target });
                        } else {
                            log::warn!("malformed command: {line}");
                        }
                    }
                    [command, included, ..] if command == "file" || command == "add" => {
                        let included = parse_path(included);
                        if included.is_empty() {
                            log::warn!("malformed command: {line}");
                        } else {
                            work.push(Work::File(PathBuf::from(included)));
                        }
                    }
                    _ => log::warn!("malformed command: {line}"),
                }
                continue;
            };
            if !visited.insert(path.clone()) {
                continue;
            }
            let content = match tokio::fs::read_to_string(&path).await {
                Ok(content) => content,
                Err(error)
                    if path == self.custom_list_path
                        && error.kind() == std::io::ErrorKind::NotFound =>
                {
                    return Ok((Vec::new(), Vec::new()));
                }
                Err(error) if path == self.custom_list_path => {
                    return Err(error)
                        .with_context(|| format!("failed to read custom list {}", path.display()));
                }
                Err(error) => {
                    log::warn!("failed to read {}: {error}", path.display());
                    continue;
                }
            };
            work.extend(content.lines().rev().map(|line| Work::Line(line.into())));
        }
        stable_deduplicate(&mut ignore_list);
        stable_deduplicate_mounts(&mut custom_mounts);
        Ok((ignore_list, custom_mounts))
    }
}

fn normalize(config: &mut ApiConfig) -> Result<(), ConfigError> {
    config.mountsource = config.mountsource.trim().into();
    for value in config
        .partitions
        .iter_mut()
        .chain(config.ignore_list.iter_mut())
    {
        *value = value.trim().into();
    }
    for mount in &mut config.custom_mounts {
        mount.source = mount.source.trim().into();
        mount.target = mount.target.trim().into();
    }
    if config.mountsource.is_empty()
        || config.partitions.iter().any(String::is_empty)
        || config.ignore_list.iter().any(String::is_empty)
        || config
            .custom_mounts
            .iter()
            .any(|mount| mount.source.is_empty() || mount.target.is_empty())
    {
        return Err(ConfigError::Invalid(
            "config values must not be empty".into(),
        ));
    }
    if config.mountsource.chars().any(char::is_control) {
        return Err(ConfigError::Invalid("invalid mount source".into()));
    }
    if config
        .partitions
        .iter()
        .any(|partition| !valid_partition_name(partition))
    {
        return Err(ConfigError::Invalid("invalid partition name".into()));
    }
    stable_deduplicate(&mut config.partitions);
    stable_deduplicate(&mut config.ignore_list);
    stable_deduplicate_mounts(&mut config.custom_mounts);
    for path in config.ignore_list.iter().chain(
        config
            .custom_mounts
            .iter()
            .flat_map(|mount| [&mount.source, &mount.target]),
    ) {
        if !safe_absolute_path(path) {
            return Err(ConfigError::Invalid(format!(
                "custom rule path must be absolute and cannot contain '..': {path:?}"
            )));
        }
        format_path(path)?;
    }
    Ok(())
}

fn valid_partition_name(value: &str) -> bool {
    !matches!(value, "." | "..")
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

fn safe_absolute_path(value: &str) -> bool {
    let path = Path::new(value);
    path.is_absolute() && !path.components().any(|part| part == Component::ParentDir)
}

fn stable_deduplicate<T: Clone + Eq + std::hash::Hash>(values: &mut Vec<T>) {
    let mut seen = FxHashSet::default();
    values.retain(|value| seen.insert(value.clone()));
}

fn stable_deduplicate_mounts(values: &mut Vec<CustomMount>) {
    let mut seen = FxHashSet::default();
    values.retain(|mount| seen.insert((mount.source.clone(), mount.target.clone())));
}

fn format_custom_list(
    ignore_list: &[String],
    custom_mounts: &[CustomMount],
) -> Result<String, ConfigError> {
    let mut lines: Vec<_> = ignore_list
        .iter()
        .map(|path| format_path(path).map(|path| format!("ignore {path}")))
        .collect::<Result<_, _>>()?;
    lines.extend(
        custom_mounts
            .iter()
            .map(|mount| {
                Ok(format!(
                    "bind {} {}",
                    format_path(&mount.source)?,
                    format_path(&mount.target)?
                ))
            })
            .collect::<Result<Vec<_>, ConfigError>>()?,
    );
    Ok(if lines.is_empty() {
        String::new()
    } else {
        lines.join("\n") + "\n"
    })
}

fn format_path(path: &str) -> Result<String, ConfigError> {
    for candidate in [path.to_string(), format!("'{path}'"), format!("\"{path}\"")] {
        let tokens = tokenize(&format!("{candidate} sentinel"));
        if tokens.len() == 2 && tokens[1] == "sentinel" && parse_path(&tokens[0]) == path {
            return Ok(candidate);
        }
    }
    Err(ConfigError::Invalid(format!(
        "path cannot be represented in custom list syntax: {path:?}"
    )))
}

fn temporary_path(path: &Path) -> PathBuf {
    let name = path.file_name().unwrap_or_default().to_string_lossy();
    let suffix = fastrand::u64(..);
    path.with_file_name(format!(".{name}.{suffix:016x}.tmp"))
}

async fn write_temporary(path: &Path, content: &[u8]) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .await?;
    let result = async {
        file.write_all(content).await?;
        file.sync_all().await
    }
    .await;
    if result.is_err() {
        let _ = tokio::fs::remove_file(path).await;
    }
    result.map_err(Into::into)
}

async fn read_optional(path: &Path) -> anyhow::Result<Option<Vec<u8>>> {
    match tokio::fs::read(path).await {
        Ok(content) => Ok(Some(content)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

async fn restore_optional(path: &Path, content: Option<Vec<u8>>) -> anyhow::Result<()> {
    let Some(content) = content else {
        return match tokio::fs::remove_file(path).await {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        };
    };
    let temporary = temporary_path(path);
    write_temporary(&temporary, &content).await?;
    if let Err(error) = tokio::fs::rename(&temporary, path).await {
        let _ = tokio::fs::remove_file(temporary).await;
        return Err(error.into());
    }
    Ok(())
}

fn parse_path(input: &str) -> String {
    if input.is_empty() {
        return String::new();
    }
    let first = input.as_bytes()[0] as char;
    let last = input.as_bytes()[input.len() - 1] as char;
    let value = if (first == '\'' && last == '"') || (first == '"' && last == '\'') {
        log::error!("mixed quotes detected in path: {input}");
        String::new()
    } else if input.len() > 1 && (first == '\'' || first == '"') && first == last {
        input[1..input.len() - 1].to_string()
    } else {
        input.to_string()
    };
    value.chars().filter(|c| !c.is_control()).collect()
}

fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quote = None;
    for ch in input.chars() {
        match in_quote {
            Some(quote) => {
                current.push(ch);
                if ch == quote {
                    in_quote = None;
                }
            }
            None if ch == '\'' || ch == '"' => {
                in_quote = Some(ch);
                current.push(ch);
            }
            None if ch.is_ascii_whitespace() => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            None => current.push(ch),
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

#[cfg(test)]
#[path = "../../../tests/unit/daemon/config.rs"]
mod tests;
