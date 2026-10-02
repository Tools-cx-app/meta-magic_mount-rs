/*
 * Copyright (C) 2026 meta-magic_mount-rs developers
 * SPDX-License-Identifier: GPL-v3
 */

export const DEFAULT_CONFIG = {
  mountsource: "KSU",
  umount: true,
  partitions: [],
  extra_mount: [],
  ignoreList: [],
  customMounts: [],
};

export const module_id = import.meta.env.MODULE_ID;

export const PATHS = {
  BINARY: "/data/adb/modules/" + module_id + "/meta-mm",
};

export const BUILTIN_PARTITIONS = [
  "vendor",
  "system_ext",
  "product",
  "odm",
] as const;
