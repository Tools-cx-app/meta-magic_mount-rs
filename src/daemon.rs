// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use std::path::Path;

use api::ApiConfig;
use serde::Deserialize;

#[derive(Deserialize)]
struct DiskConfig {
    mountsource: String,
    umount: bool,
    partitions: Vec<String>,
}

pub fn load_config(config_path: &Path, custom_path: &Path) -> anyhow::Result<ApiConfig> {
    let disk: DiskConfig = toml::from_str(&std::fs::read_to_string(config_path)?)?;
    let (ignore_list, custom_mounts) = crate::parser::load_custom(custom_path);
    Ok(ApiConfig {
        mountsource: disk.mountsource,
        umount: disk.umount,
        partitions: disk.partitions,
        ignore_list,
        custom_mounts,
    })
}

#[cfg(test)]
mod tests {
    use super::load_config;

    #[test]
    fn loads_persistent_config_without_daemon() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        let custom_path = dir.path().join("custom");
        std::fs::write(
            &config_path,
            "mountsource = \"KSU\"\numount = false\npartitions = [\"vendor\"]\n",
        )
        .unwrap();
        std::fs::write(&custom_path, "ignore /ignored\nbind /source /target\n").unwrap();

        let config = load_config(&config_path, &custom_path).unwrap();
        assert_eq!(config.mountsource, "KSU");
        assert_eq!(config.partitions, ["vendor"]);
        assert_eq!(config.ignore_list, ["/ignored"]);
        assert_eq!(config.custom_mounts[0].source, "/source");
        assert_eq!(config.custom_mounts[0].target, "/target");
    }
}
