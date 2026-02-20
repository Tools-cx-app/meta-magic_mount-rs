// Copyright 2025 Magic Mount-rs Authors
// SPDX-License-Identifier: GPL-3.0-or-later

pub const MODULE_PATH: &str = "/data/adb/modules/";

// utils
pub const SELINUX_XATTR: &str = "security.selinux";
// magic_mount
pub const DISABLE_FILE_NAME: &str = "disable";
pub const REMOVE_FILE_NAME: &str = "remove";
pub const SKIP_MOUNT_FILE_NAME: &str = "skip_mount";
pub const REPLACE_DIR_XATTR: &str = "trusted.overlay.opaque";
pub const REPLACE_DIR_FILE_NAME: &str = ".replace";
pub const FORCE_DISABLE_UMOUNT: &str = "/data/adb/magic_mount/.force_disable_umount";

// config
pub const CONFIG_FILE: &str = "/data/adb/magic_mount/config.toml";
