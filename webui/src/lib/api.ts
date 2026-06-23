/*
 * Copyright (C) 2026 meta-magic_mount-rs developers
 * SPDX-License-Identifier: GPL-v3
 */

import type { AppAPI } from "./types";
import type { KsuExec } from "./api.client";
import { MockAPI } from "./api.mock";
import { createRealAPI } from "./api.client";

let ksuExec: KsuExec | null = null;

try {
  const ksu = await import("kernelsu").catch(() => null);
  ksuExec = ksu ? ksu.exec : null;
} catch {}

export const API: AppAPI =
  import.meta.env.DEV || !ksuExec ? MockAPI : createRealAPI(ksuExec);
