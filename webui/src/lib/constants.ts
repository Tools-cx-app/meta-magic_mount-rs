/*
 * Copyright (C) 2026 meta-magic_mount-rs developers
 * SPDX-License-Identifier: Apache-2.0
 */

export const DEFAULT_CONFIG = {
  mountsource: "KSU",
  umount: true,
  partitions: [],
  ignoreList: [],
  customMounts: [],
};

export const PATHS = {
  BINARY: "/data/adb/modules/magic_mount_rs/meta-mm",
};

export const BUILTIN_PARTITIONS = [
  "vendor",
  "system_ext",
  "product",
  "odm",
] as const;
