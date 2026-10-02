/*
 * Copyright (C) 2026 meta-magic_mount-rs developers
 * SPDX-License-Identifier: GPL-v3
 */

export function isValidExtraMount(name: string): boolean {
  return (
    name.length > 0 &&
    ![".", "..", "system"].includes(name) &&
    !name.includes("/") &&
    !name.includes("\0")
  );
}
