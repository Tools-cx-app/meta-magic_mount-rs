// Copyright (C) 2026 meta-magic_mount-rs developers
// SPDX-License-Identifier: Apache-2.0
use const_format::concatcp;

const ADB: &str = "/data/adb";
pub const MODULE_PATH: &str = concatcp!(ADB, "/modules");
const MAGIC_MOUNT: &str = concatcp!(ADB, "/magic_mount");
pub const CUSTOM_LIST_PATH: &str = concatcp!(MAGIC_MOUNT, "/custom");
pub const SELINUX_XATTR: &str = "security.selinux";
pub const DISABLE_FILE_NAME: &str = "disable";
pub const REMOVE_FILE_NAME: &str = "remove";
pub const SKIP_MOUNT_FILE_NAME: &str = "skip_mount";
pub const REPLACE_DIR_XATTR: &str = "trusted.overlay.opaque";
pub const REPLACE_DIR_FILE_NAME: &str = ".replace";
pub const CONFIG_FILE: &str = concatcp!(MAGIC_MOUNT, "/config.toml");
pub const MODULE_PROP: &str = concatcp!(MODULE_PATH, "/magic_mount_rs/module.prop");
