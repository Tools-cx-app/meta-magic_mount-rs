/*
 * Copyright (C) 2026 Tools-cx-app <localhost.hutao@gmail.com>
 * SPDX-License-Identifier: Apache-2.0
 */

import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";

// https://vite.dev/config/
export default defineConfig({
  base: "./",
  build: {
    outDir: "../module/webroot",
    target: "esnext",
  },
  plugins: [vue()],
});
