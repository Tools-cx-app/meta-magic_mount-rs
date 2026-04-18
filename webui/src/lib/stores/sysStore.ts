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

import type { DeviceInfo, SystemInfo } from "../../types";
import { API } from "../api";
import { uiStore } from "./uiStore";

function createSysStore() {
  const [device, setDevice] = createSignal<DeviceInfo>({
    model: "-",
  });
  const [version, setVersion] = createSignal("...");
  const [systemInfo, setSystemInfo] = createSignal<SystemInfo>({
    kernel: "-",
    selinux: "-",
  });
  const [loading, setLoading] = createSignal(false);
  let pendingLoad: Promise<void> | null = null;
  let hasLoaded = false;

  async function loadStatus() {
    if (pendingLoad) {
      return pendingLoad;
    }

    setLoading(true);
    pendingLoad = (async () => {
      try {
        const [baseDevice, nextVersion, info] = await Promise.all([
          API.getDeviceStatus(),
          API.getVersion(),
          API.getSystemInfo(),
        ]);

        setDevice(baseDevice);
        setVersion(nextVersion);
        setSystemInfo(info);
        hasLoaded = true;
      } catch {
        uiStore.showToast(
          uiStore.L.status.loadError ?? "Failed to load system status",
          "error",
        );
      } finally {
        setLoading(false);
        pendingLoad = null;
      }
    })();

    return pendingLoad;
  }

  function ensureStatusLoaded() {
    if (hasLoaded) {
      return Promise.resolve();
    }

    return loadStatus();
  }

  async function rebootDevice() {
    try {
      await API.reboot();
    } catch {
      uiStore.showToast(
        uiStore.L.common.rebootFailed ?? "Reboot failed",
        "error",
      );
    }
  }

  return {
    get device() {
      return device();
    },
    get version() {
      return version();
    },
    get systemInfo() {
      return systemInfo();
    },
    get loading() {
      return loading();
    },
    get hasLoaded() {
      return hasLoaded;
    },
    ensureStatusLoaded,
    loadStatus,
    rebootDevice,
  };
}

export const sysStore = createRoot(createSysStore);
