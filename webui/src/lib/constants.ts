/*
 * Copyright (C) 2026 meta-magic_mount-rs developers
 * SPDX-License-Identifier: GPL-v3
 */

export const DEFAULT_CONFIG = {
  mountsource: "KSU",
  umount: true,
  partitions: [],
  ignoreList: [],
  customMounts: [],
};

export const module_id = import.meta.env?.MODULE_ID ?? "test";

export const PATHS = {
  CONNECTION: "/data/adb/magic_mount/daemon.json",
};

export const BUILTIN_PARTITIONS = [
  "vendor",
  "system_ext",
  "product",
  "odm",
] as const;
