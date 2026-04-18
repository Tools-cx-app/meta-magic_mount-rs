// Copyright (C) 2026 Tools-cx-app <localhost.hutao@gmail.com>
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

pub const MODULE_PATH: &str = "/data/adb/modules/";
pub const IGNORE_LIST_PATH: &str = "/data/adb/magic_mount/ignore.list";
pub const SELINUX_XATTR: &str = "security.selinux";
pub const DISABLE_FILE_NAME: &str = "disable";
pub const REMOVE_FILE_NAME: &str = "remove";
pub const SKIP_MOUNT_FILE_NAME: &str = "skip_mount";
pub const REPLACE_DIR_XATTR: &str = "trusted.overlay.opaque";
pub const REPLACE_DIR_FILE_NAME: &str = ".replace";
pub const CONFIG_FILE: &str = "/data/adb/magic_mount/config.toml";
pub const MODULE_PROP: &str = "/data/adb/modules/magic_mount_rs/module.prop";
