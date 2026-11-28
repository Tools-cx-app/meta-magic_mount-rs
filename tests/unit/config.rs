// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use super::*;

#[test]
fn extra_mount_selects_only_missing_system_entries() {
    use std::os::unix::fs::symlink;
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    fs::create_dir(root.join("system")).unwrap();
    for name in [
        "my_product",
        "directory",
        "file",
        "link",
        "dangling",
        "loop",
    ] {
        fs::create_dir(root.join(name)).unwrap();
    }
    fs::create_dir(root.join("system/directory")).unwrap();
    fs::write(root.join("system/file"), "").unwrap();
    fs::write(root.join("not_directory"), "").unwrap();
    symlink(root.join("link"), root.join("system/link")).unwrap();
    symlink("missing", root.join("system/dangling")).unwrap();
    symlink("loop", root.join("system/loop")).unwrap();
    let mut config = Config::default();
    config.extra_mount = [
        "my_product",
        "my_product",
        "directory",
        "file",
        "link",
        "dangling",
        "loop",
        "missing",
        "not_directory",
        "",
        ".",
        "..",
        "system",
        "/my_product",
        "../my_product",
        "my_product/",
        "a/b",
        "a\0b",
    ]
    .map(str::to_string)
    .to_vec();
    assert_eq!(config.extra_mount_partitions(root), ["my_product"]);
    fs::remove_dir_all(root.join("system")).unwrap();
    symlink("system", root.join("system")).unwrap();
    assert!(config.extra_mount_partitions(root).is_empty());
}

#[test]
fn test_config_save_and_load_flow() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let config_file_path = tmp_dir.path().join("config.toml");
    let mut config = Config::default();
    config.partitions = vec!["vendor".to_string()];
    config.umount = true;
    assert!(config.save(&config_file_path).is_ok());

    let loaded_config = Config::load(&config_file_path).unwrap();
    assert_eq!(loaded_config.mountsource, "KSU");
    assert_eq!(loaded_config.partitions, vec!["vendor"]);
    assert!(loaded_config.umount);
}

#[test]
fn test_load_or_default_fallback() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let config = Config::load_or_default(tmp_dir.path().join("missing.toml"));
    assert_eq!(config.mountsource, "KSU");
    assert!(!config.umount);
}
