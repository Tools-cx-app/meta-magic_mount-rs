// Copyright (C) 2026 Tools-cx-app <localhost.hutao@gmail.com>
// SPDX-License-Identifier: Apache-2.0

pub const MODULE_PATH: &str = "/data/adb/modules/";
pub const CUSTOM_LIST_PATH: &str = "/data/adb/magic_mount/custom";
pub const SELINUX_XATTR: &str = "security.selinux";
pub const DISABLE_FILE_NAME: &str = "disable";
pub const REMOVE_FILE_NAME: &str = "remove";
pub const SKIP_MOUNT_FILE_NAME: &str = "skip_mount";
pub const REPLACE_DIR_XATTR: &str = "trusted.overlay.opaque";
pub const REPLACE_DIR_FILE_NAME: &str = ".replace";
pub const CONFIG_FILE: &str = "/data/adb/magic_mount/config.toml";
#[cfg(target_arch = "arm")]
pub const LIBRARY: &str = "/data/adb/modules/magic_mount_rs/libs/arm/libchecker.so";
#[cfg(target_arch = "aarch64")]
pub const LIBRARY: &str = "/data/adb/modules/magic_mount_rs/libs/arm64-v8a/libchecker.so";
pub const SELF_MODULE_PATH: &str = "/data/adb/modules/magic_mount_rs";
