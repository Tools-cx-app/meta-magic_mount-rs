/*
 * Copyright (C) 2026 meta-magic_mount-rs developers
 * SPDX-License-Identifier: GPL-v3
 */

import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

let moduleid = process.env.MODULE_ID;

if (!moduleid) {
  moduleid = "test";
}

export default defineConfig({
  base: "./",
  build: {
    outDir: "../module/webroot",
    target: "esnext",
    chunkSizeWarningLimit: 1000,
  },
  define: {
    "import.meta.env.MODULE_ID": JSON.stringify(moduleid),
  },
  plugins: [vue()],
});
