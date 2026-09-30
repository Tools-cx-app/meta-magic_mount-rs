// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use super::*;

#[test]
fn test_decode_hex_valid_cases() {
    let input_lowercase = "74657374";
    let input_uppercase = "54455354";
    let input_extremes = "00ff00ff";

    assert_eq!(decode(input_lowercase).unwrap(), b"test".to_vec());
    assert_eq!(decode(input_uppercase).unwrap(), b"TEST".to_vec());
    assert_eq!(decode("").unwrap(), Vec::<u8>::new());
    assert_eq!(
        decode(input_extremes).unwrap(),
        vec![0x00, 0xFF, 0x00, 0xFF]
    );
}

#[test]
fn test_decode_hex_invalid_length_error() {
    assert!(decode("a").is_err());
    assert!(decode("12345").is_err());
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
    assert_eq!(config.partitions, vec!["system", "product"]);
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
    let api_config = config.into_api(vec!["/data/local/tmp".to_string()], vec![]);
    assert_eq!(api_config.mountsource, "KSU");
    assert_eq!(api_config.partitions, vec!["system"]);
    assert!(!api_config.umount);
    assert!(api_config.disable_umount);
    assert_eq!(api_config.ignore_list[0], "/data/local/tmp");
}
