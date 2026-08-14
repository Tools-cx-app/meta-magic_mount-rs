// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use std::{fmt, sync::OnceLock};

use api::ApiConfig;

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
