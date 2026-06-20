/*
 * Copyright (C) 2026 Tools-cx-app <localhost.hutao@gmail.com>
 * SPDX-License-Identifier: Apache-2.0
 */

import { ref } from "vue";
import type { Module } from "../types";
import { API } from "../api";
import { uiStore } from "./uiStore";

const modules = ref<Module[]>([]);
const loading = ref(false);
let pendingLoad: Promise<void> | null = null;
let hasLoaded = false;

async function loadModules() {
  if (pendingLoad) {
    return pendingLoad;
  }

  loading.value = true;
  pendingLoad = (async () => {
    try {
      const data = await API.scanModules();
      modules.value = [...data];
      hasLoaded = true;
    } catch {
      uiStore.showToast("Failed to scan modules");
    } finally {
      loading.value = false;
      pendingLoad = null;
    }
  })();

  return pendingLoad;
}

function ensureModulesLoaded() {
  if (hasLoaded) {
    return Promise.resolve();
  }

  return loadModules();
}

export const moduleStore = {
  get modules() {
    return modules.value;
  },
  get loading() {
    return loading.value;
  },
  get hasLoaded() {
    return hasLoaded;
  },
  ensureModulesLoaded,
  loadModules,
};
