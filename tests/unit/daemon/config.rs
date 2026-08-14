// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use api::{ApiConfig, CustomMount};

use super::{ConfigError, Store, format_path, parse_path};

#[test]
fn empty_path_parses_as_empty() {
    assert_eq!(parse_path(""), "");
}

#[tokio::test]
async fn save_trims_and_stably_deduplicates() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::new(dir.path().join("config.toml"), dir.path().join("custom"));
    store
        .save(ApiConfig {
            mountsource: " KSU ".into(),
            umount: true,
            partitions: vec![" vendor ".into(), "vendor".into()],
            ignore_list: vec![" /ignored ".into(), "/ignored".into()],
            custom_mounts: vec![
                CustomMount {
                    source: " /a ".into(),
                    target: " /b ".into(),
                },
                CustomMount {
                    source: "/a".into(),
                    target: "/b".into(),
                },
            ],
        })
        .await
        .unwrap();

    let loaded = store.load().await.unwrap();
    assert_eq!(loaded.mountsource, "KSU");
    assert_eq!(loaded.partitions, ["vendor"]);
    assert_eq!(loaded.ignore_list, ["/ignored"]);
    assert_eq!(
        loaded.custom_mounts,
        [CustomMount {
            source: "/a".into(),
            target: "/b".into(),
        }]
    );
}

#[tokio::test]
async fn save_rejects_empty_values_without_overwriting() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config.toml");
    let store = Store::new(&config, dir.path().join("custom"));
    tokio::fs::write(
        &config,
        "mountsource = \"KSU\"\npartitions = []\numount = false\n",
    )
    .await
    .unwrap();

    let invalid = ApiConfig {
        mountsource: "".into(),
        umount: true,
        partitions: vec![],
        ignore_list: vec![],
        custom_mounts: vec![],
    };
    assert!(matches!(
        store.save(invalid).await,
        Err(ConfigError::Invalid(_))
    ));
    assert!(
        tokio::fs::read_to_string(config)
            .await
            .unwrap()
            .contains("umount = false")
    );
}

#[tokio::test]
async fn save_rejects_unsafe_partitions_and_custom_paths() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("config.toml");
    let store = Store::new(&config_path, dir.path().join("custom"));
    tokio::fs::write(
        &config_path,
        "mountsource = \"KSU\"\npartitions = []\numount = false\n",
    )
    .await
    .unwrap();
    let base = ApiConfig {
        mountsource: "KSU".into(),
        umount: false,
        partitions: vec![],
        ignore_list: vec![],
        custom_mounts: vec![],
    };

    for invalid in [
        ApiConfig {
            partitions: vec!["../vendor".into()],
            ..base.clone()
        },
        ApiConfig {
            ignore_list: vec!["relative/path".into()],
            ..base.clone()
        },
        ApiConfig {
            custom_mounts: vec![CustomMount {
                source: "/safe/source".into(),
                target: "/system/../vendor/target".into(),
            }],
            ..base.clone()
        },
    ] {
        assert!(matches!(
            store.save(invalid).await,
            Err(ConfigError::Invalid(_))
        ));
    }
    assert!(
        tokio::fs::read_to_string(config_path)
            .await
            .unwrap()
            .contains("umount = false")
    );
}

#[tokio::test]
async fn failed_custom_rule_commit_restores_config() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config.toml");
    let custom = dir.path().join("custom");
    tokio::fs::write(
        &config,
        "mountsource = \"KSU\"\npartitions = []\numount = false\n",
    )
    .await
    .unwrap();
    tokio::fs::create_dir(&custom).await.unwrap();

    let result = Store::new(&config, custom)
        .save(ApiConfig {
            mountsource: "APatch".into(),
            umount: true,
            partitions: vec![],
            ignore_list: vec!["/ignored".into()],
            custom_mounts: vec![],
        })
        .await;

    assert!(result.is_err());
    let restored = tokio::fs::read_to_string(config).await.unwrap();
    assert!(restored.contains("mountsource = \"KSU\""));
    assert!(restored.contains("umount = false"));
}

#[tokio::test]
async fn missing_custom_list_loads_as_empty() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config.toml");
    tokio::fs::write(
        &config,
        "mountsource = \"KSU\"\npartitions = []\numount = false\n",
    )
    .await
    .unwrap();
    let loaded = Store::new(config, dir.path().join("missing"))
        .load()
        .await
        .unwrap();
    assert!(loaded.ignore_list.is_empty());
    assert!(loaded.custom_mounts.is_empty());
}

#[tokio::test]
async fn load_parses_quoted_paths_and_included_files_once() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config.toml");
    let custom = dir.path().join("custom");
    let included = dir.path().join("included");
    tokio::fs::write(
        &config,
        "mountsource = \"KSU\"\npartitions = []\numount = false\n",
    )
    .await
    .unwrap();
    tokio::fs::write(&included, "ignore /included\n")
        .await
        .unwrap();
    tokio::fs::write(
        &custom,
        format!(
            "ignore \"/path with spaces\"\nbind '/source path' /target\nfile {}\nadd {}\n",
            included.display(),
            included.display()
        ),
    )
    .await
    .unwrap();

    let loaded = Store::new(config, custom).load().await.unwrap();
    assert_eq!(loaded.ignore_list, ["/path with spaces", "/included"]);
    assert_eq!(
        loaded.custom_mounts,
        [CustomMount {
            source: "/source path".into(),
            target: "/target".into(),
        }]
    );
}

#[tokio::test]
async fn load_ignores_trailing_command_tokens() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config.toml");
    let custom = dir.path().join("custom");
    let included = dir.path().join("included");
    tokio::fs::write(
        &config,
        "mountsource = \"KSU\"\npartitions = []\numount = false\n",
    )
    .await
    .unwrap();
    tokio::fs::write(&included, "ignore /included trailing\n")
        .await
        .unwrap();
    tokio::fs::write(
            &custom,
            format!(
                "ignore /ignored trailing\nbind /source /target trailing\nfile {} trailing\nadd {} trailing\n",
                included.display(),
                included.display()
            ),
        )
        .await
        .unwrap();

    let loaded = Store::new(config, custom).load().await.unwrap();
    assert_eq!(loaded.ignore_list, ["/ignored", "/included"]);
    assert_eq!(
        loaded.custom_mounts,
        [CustomMount {
            source: "/source".into(),
            target: "/target".into(),
        }]
    );
}

#[tokio::test]
async fn quote_containing_paths_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::new(dir.path().join("config.toml"), dir.path().join("custom"));
    let expected = ApiConfig {
        mountsource: "KSU".into(),
        umount: false,
        partitions: vec![],
        ignore_list: vec!["/single ' quote".into(), "/double \" quote".into()],
        custom_mounts: vec![CustomMount {
            source: "/source ' quote".into(),
            target: "/target \" quote".into(),
        }],
    };

    store.save(expected.clone()).await.unwrap();

    assert_eq!(store.load().await.unwrap(), expected);
}

#[test]
fn single_character_quotes_have_a_representable_form() {
    assert_eq!(format_path("'").unwrap(), "\"'\"");
    assert_eq!(format_path("\"").unwrap(), "'\"'");
}

#[tokio::test]
async fn unrepresentable_path_does_not_overwrite_existing_files() {
    let dir = tempfile::tempdir().unwrap();
    let config = dir.path().join("config.toml");
    let custom = dir.path().join("custom");
    tokio::fs::write(&config, "existing config").await.unwrap();
    tokio::fs::write(&custom, "existing custom").await.unwrap();
    let store = Store::new(&config, &custom);

    let result = store
        .save(ApiConfig {
            mountsource: "KSU".into(),
            umount: false,
            partitions: vec![],
            ignore_list: vec!["/both ' and \" quotes".into()],
            custom_mounts: vec![],
        })
        .await;

    assert!(matches!(result, Err(ConfigError::Invalid(_))));
    assert_eq!(
        tokio::fs::read_to_string(config).await.unwrap(),
        "existing config"
    );
    assert_eq!(
        tokio::fs::read_to_string(custom).await.unwrap(),
        "existing custom"
    );
}
