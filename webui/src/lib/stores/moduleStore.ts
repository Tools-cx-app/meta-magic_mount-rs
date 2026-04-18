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
