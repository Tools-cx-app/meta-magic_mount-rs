/**
 * Copyright 2025 Magic Mount-rs Authors
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import { ICONS } from "./constants";

export const TABS = [
  { id: "status", icon: ICONS.home },
  { id: "config", icon: ICONS.settings },
  { id: "modules", icon: ICONS.modules },
  { id: "info", icon: ICONS.info },
] as const;

export type TabId = (typeof TABS)[number]["id"];
