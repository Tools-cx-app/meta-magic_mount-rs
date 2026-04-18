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

import { createRoot, createSignal } from "solid-js";
import { createStore, reconcile } from "solid-js/store";

import type { AppConfig } from "../../types";
import { API } from "../api";
import { DEFAULT_CONFIG } from "../constants";
import { uiStore } from "./uiStore";

function createConfigStore() {
  const [config, setConfigStore] = createStore<AppConfig>(DEFAULT_CONFIG);
  const [loading, setLoading] = createSignal(false);
  const [saving, setSaving] = createSignal(false);

  function setConfig(next: AppConfig) {
    setConfigStore(reconcile(next));
  }

  async function loadConfig() {
    setLoading(true);
    try {
      setConfigStore(reconcile(await API.loadConfig()));
    } catch {
      uiStore.showToast(
        uiStore.L.config.loadError ?? "Failed to load config",
        "error",
      );
    }
    setLoading(false);
  }

  async function saveConfig() {
    setSaving(true);
    try {
      await API.saveConfig(config);
      uiStore.showToast(
        uiStore.L.config.saveSuccess ?? "Configuration saved",
        "success",
      );
    } catch {
      uiStore.showToast(
        uiStore.L.config.saveFailed ?? "Failed to save configuration",
        "error",
      );
    }
    setSaving(false);
  }

  return {
    get config() {
      return config;
    },
    setConfig,
    get loading() {
      return loading();
    },
    get saving() {
      return saving();
    },
    loadConfig,
    saveConfig,
  };
}

export const configStore = createRoot(createConfigStore);
