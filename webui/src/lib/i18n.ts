/*
 * Copyright (C) 2026 Tools-cx-app <localhost.hutao@gmail.com>
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
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
