import { createMemo, createRoot, createSignal } from "solid-js";
import { createStore, reconcile } from "solid-js/store";

import type { ModeStats, Module } from "../../types";
import { API } from "../api";
import { normalizeModuleMode } from "../moduleMode";
import { uiStore } from "./uiStore";

function createModuleStore() {
  const [modules, setModulesStore] = createStore<Module[]>([]);
  const [loading, setLoading] = createSignal(false);
  let pendingLoad: Promise<void> | null = null;
  let hasLoaded = false;

  const normalizeModule = (module: Module): Module => ({
    ...module,
    mode: normalizeModuleMode(module.mode),
    rules: {
      ...module.rules,
      default_mode: normalizeModuleMode(module.rules.default_mode),
    },
  });

  const modeStats = createMemo((): ModeStats => {
    const stats = { overlay: 0, magic: 0 };

    for (const module of modules) {
      if (!module.is_mounted) {
        continue;
      }

      const mode = normalizeModuleMode(module.mode);
      if (mode === "overlay") {
        stats.overlay++;
      } else if (mode === "magic") {
        stats.magic++;
      }
    }

    return stats;
  });

  async function loadModules() {
    if (pendingLoad) {
      return pendingLoad;
    }

    setLoading(true);
    pendingLoad = (async () => {
      try {
        const data = (await API.scanModules()).map(normalizeModule);
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

  function setModules(next: Module[]) {
    setModulesStore(reconcile(next.map(normalizeModule)));
  }

  return {
    get modules() {
      return modules;
    },
    setModules,
    get loading() {
      return loading();
    },
    get hasLoaded() {
      return hasLoaded;
    },
    get modeStats() {
      return modeStats();
    },
    ensureModulesLoaded,
    loadModules,
  };
}

export const moduleStore = createRoot(createModuleStore);
