/*
 * Copyright (C) 2026 meta-magic_mount-rs developers
 * SPDX-License-Identifier: Apache-2.0
 */

import { defineConfig } from "vite";
import solid from "vite-plugin-solid";

export default defineConfig({
  base: "./",
  build: {
    outDir: "../module/webroot",
  },
  plugins: [solid()],
});
