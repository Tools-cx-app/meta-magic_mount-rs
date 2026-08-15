// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use std::{fmt, path::Path, sync::OnceLock};

use api::{ApiConfig, CustomMount};
use rustc_hash::FxHashSet;

pub static COMMAND_LIST: OnceLock<Vec<MountType>> = OnceLock::new();

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MountType {
    Mount { source: String, target: String },
    Ignore { source: String },
}

pub fn init_from_config(config: &ApiConfig) -> anyhow::Result<()> {
    let mut commands = config
        .ignore_list
        .iter()
        .cloned()
        .map(|source| MountType::Ignore { source })
        .collect::<Vec<_>>();
    commands.extend(
        config
            .custom_mounts
            .iter()
            .cloned()
            .map(|mount| MountType::Mount {
                source: mount.source,
                target: mount.target,
            }),
    );
    COMMAND_LIST
        .set(commands)
        .map_err(|_| anyhow::anyhow!("mount command list is already initialized"))
}

pub fn load_custom(path: &Path) -> (Vec<String>, Vec<CustomMount>) {
    enum Work {
        File(std::path::PathBuf),
        Line(String),
    }

    let mut ignores = Vec::new();
    let mut mounts = Vec::new();
    let mut visited = FxHashSet::default();
    let mut work = vec![Work::File(path.into())];
    while let Some(item) = work.pop() {
        match item {
            Work::File(path) if visited.insert(path.clone()) => {
                if let Ok(content) = std::fs::read_to_string(path) {
                    work.extend(content.lines().rev().map(|line| Work::Line(line.into())));
                }
            }
            Work::Line(line) => {
                let tokens = tokenize(line.trim());
                match tokens.as_slice() {
                    [command, source, ..] if command == "ignore" => ignores.push(source.clone()),
                    [command, source, target, ..] if command == "bind" => {
                        mounts.push(CustomMount {
                            source: source.clone(),
                            target: target.clone(),
                        });
                    }
                    [command, included, ..] if command == "file" || command == "add" => {
                        work.push(Work::File(included.into()));
                    }
                    _ => {}
                }
            }
            Work::File(_) => {}
        }
    }
    (ignores, mounts)
}

fn tokenize(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    for ch in input.chars() {
        match quote {
            Some(end) if ch == end => quote = None,
            None if ch == '\'' || ch == '"' => quote = Some(ch),
            None if ch.is_ascii_whitespace() => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            Some(_) | None => current.push(ch),
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}

impl fmt::Display for MountType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mount { source, target } => f.write_str(&format!("{source} -> {target}")),
            Self::Ignore { source } => f.write_str(&format!("missing {source}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use api::{ApiConfig, CustomMount};

    use super::{COMMAND_LIST, MountType, init_from_config};

    #[test]
    fn initializes_commands_from_daemon_config() {
        init_from_config(&ApiConfig {
            mountsource: "KSU".into(),
            umount: false,
            partitions: vec![],
            ignore_list: vec!["/ignored".into()],
            custom_mounts: vec![CustomMount {
                source: "/source".into(),
                target: "/target".into(),
            }],
        })
        .unwrap();
        assert_eq!(
            COMMAND_LIST.get().unwrap(),
            &[
                MountType::Ignore {
                    source: "/ignored".into()
                },
                MountType::Mount {
                    source: "/source".into(),
                    target: "/target".into()
                }
            ]
        );
    }
}
