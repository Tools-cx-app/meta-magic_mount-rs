// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use super::*;

#[test]
fn extra_mount_config_round_trip() {
    let mut config: Config =
        toml::from_str("mountsource = 'APatch'\npartitions = []\numount = true").unwrap();
    assert_eq!(
        serde_json::to_value(&config).unwrap()["extra_mount"],
        json!([])
    );
    config
        .apply_api_payload(serde_json::from_value(json!({"extra_mount": ["my_product"]})).unwrap());
    let tmp = tempfile::tempdir().unwrap();
    config.save(tmp.path().join("config.toml")).unwrap();
    let mut config = Config::load(tmp.path().join("config.toml")).unwrap();
    config.apply_api_payload(serde_json::from_value(json!({})).unwrap());
    assert_eq!(
        serde_json::to_value(config.into_api()).unwrap()["extra_mount"],
        json!(["my_product"])
    );
}

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
fn test_apply_api_payload() {
    let mut config = Config::default();
    let payload = ApiConfigPayload {
        mountsource: Some("APatch".to_string()),
        partitions: Some(vec!["system".to_string(), "product".to_string()]),
        extra_mount: None,
        umount: Some(true),
        disable_umount: None,
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
        extra_mount: vec![],
        umount: true,
    };
    let payload = ApiConfigPayload {
        mountsource: None,
        partitions: None,
        extra_mount: None,
        umount: None,
        disable_umount: Some(true),
    };

    config.apply_api_payload(payload);
    assert!(!config.umount);
}

#[test]
fn test_into_api_struct_mapping() {
    let config = Config {
        mountsource: "KSU".to_string(),
        partitions: vec!["system".to_string()],
        extra_mount: vec![],
        umount: false,
    };
    let api_config = config.into_api();
    assert_eq!(api_config.mountsource, "KSU");
    assert_eq!(api_config.partitions, vec!["system"]);
    assert!(!api_config.umount);
    assert!(api_config.disable_umount);
    assert_eq!(
        serde_json::to_value(api_config).unwrap(),
        json!({
            "moduledir": paths::MODULE_PATH.trim_end_matches('/'),
            "mountsource": "KSU",
            "partitions": ["system"],
            "extra_mount": [],
            "umount": false,
            "disable_umount": true,
        })
    );
}
