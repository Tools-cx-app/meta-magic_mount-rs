// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use super::*;

#[test]
fn test_config_load() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let config_file_path = tmp_dir.path().join("config.toml");
    fs::write(
        &config_file_path,
        "mountsource = \"KSU\"\npartitions = [\"vendor\"]\numount = true\n",
    )
    .unwrap();

    let loaded_config = Config::load(&config_file_path).unwrap();
    assert_eq!(loaded_config.mountsource, "KSU");
    assert_eq!(loaded_config.partitions, vec!["vendor"]);
    assert!(loaded_config.umount);
}
