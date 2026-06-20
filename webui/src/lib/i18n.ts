/*
 * Copyright (C) 2026 meta-magic_mount-rs developers
 * SPDX-License-Identifier: Apache-2.0
 */

import type en from "../locales/en.json";

export type Locale = typeof en;

const localeModules = import.meta.glob("../locales/*.json", { eager: true });

export const locales: Record<string, Locale> = Object.fromEntries(
  Object.entries(localeModules).map(([path, mod]) => {
    const code = path.match(/\/([^/]+)\.json$/)?.[1] ?? "en";

    return [code, (mod as { default: Locale }).default];
  }),
);

export const availableLanguages = Object.entries(locales)
  .map(([code, locale]) => ({
    code,
    name: locale.lang.display || code.toUpperCase(),
  }))
  .sort((a, b) => {
    if (a.code === "en") {
      return -1;
    }
    if (b.code === "en") {
      return 1;
    }

    return a.name.localeCompare(b.name);
  });
