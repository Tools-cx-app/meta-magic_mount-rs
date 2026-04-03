import { createRoot, createSignal } from "solid-js";
import { createStore, reconcile } from "solid-js/store";

import type { Module } from "../../types";
import { API } from "../api";
import { uiStore } from "./uiStore";

function createModuleStore() {
  const [modules, setModulesStore] = createStore<Module[]>([]);
  const [loading, setLoading] = createSignal(false);
  let pendingLoad: Promise<void> | null = null;
  let hasLoaded = false;

  async function loadModules() {
    if (pendingLoad) {
      return pendingLoad;
    }

    setLoading(true);
    pendingLoad = (async () => {
      try {
        const data = await API.scanModules();
        setModulesStore(reconcile(data));
        hasLoaded = true;
      } catch {
        uiStore.showToast(
          uiStore.L.modules.scanError ?? "Failed to scan modules",
          "error",
        );
      } finally {
        setLoading(false);
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

  return {
    get modules() {
      return modules;
    },
    get loading() {
      return loading();
    },
    get hasLoaded() {
      return hasLoaded;
    },
    ensureModulesLoaded,
    loadModules,
  };
}

export const moduleStore = createRoot(createModuleStore);
