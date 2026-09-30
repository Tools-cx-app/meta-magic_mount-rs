// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: GPL-v3

use super::*;
use clap::error::ErrorKind;

#[test]
fn preserves_existing_commands_and_default_mount() {
    assert!(
        Cli::try_parse_from(["magic_mount_rs"])
            .unwrap()
            .command
            .is_none()
    );
    for (name, expected) in [
        ("show-config", Commands::ShowConfig),
        ("emulated-soft-reboot", Commands::EmulatedSoftReboot),
        ("gen-config", Commands::GenConfig),
        ("modules", Commands::Modules),
        ("version", Commands::Version),
    ] {
        assert_eq!(
            Cli::try_parse_from(["magic_mount_rs", name])
                .unwrap()
                .command,
            Some(expected)
        );
    }
    assert_eq!(
        Cli::try_parse_from(["magic_mount_rs", "save-config", "--payload", "7b7d"])
            .unwrap()
            .command,
        Some(Commands::SaveConfig {
            payload: "7b7d".into()
        })
    );
}

#[test]
fn rejects_invalid_arguments() {
    for args in [
        vec!["magic_mount_rs", "unknown"],
        vec!["magic_mount_rs", "save-config"],
        vec!["magic_mount_rs", "save-config", "--payload"],
        vec!["magic_mount_rs", "modules", "extra"],
    ] {
        assert!(Cli::try_parse_from(args).is_err());
    }
}

#[test]
fn offers_help_without_a_runtime() {
    for args in [
        vec!["magic_mount_rs", "--help"],
        vec!["magic_mount_rs", "save-config", "--help"],
    ] {
        assert_eq!(
            Cli::try_parse_from(args).unwrap_err().kind(),
            ErrorKind::DisplayHelp
        );
    }
}
