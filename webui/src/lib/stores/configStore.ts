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
