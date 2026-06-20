/*
 * Copyright (C) 2026 meta-magic_mount-rs developers
 * SPDX-License-Identifier: Apache-2.0
 */

import { ref } from "vue";
import type { DeviceInfo, SystemInfo } from "../types";
import { API } from "../api";
import { uiStore } from "./uiStore";

const device = ref<DeviceInfo>({
  model: "-",
});
const version = ref("...");
const systemInfo = ref<SystemInfo>({
  kernel: "-",
  selinux: "-",
});
const loading = ref(false);
let pendingLoad: Promise<void> | null = null;
let hasLoaded = false;

async function loadStatus() {
  if (pendingLoad) {
    return pendingLoad;
  }

  loading.value = true;
  pendingLoad = (async () => {
    try {
      const [baseDevice, nextVersion, info] = await Promise.all([
        API.getDeviceStatus(),
        API.getVersion(),
        API.getSystemInfo(),
      ]);

      device.value = baseDevice;
      version.value = nextVersion;
      systemInfo.value = info;
      hasLoaded = true;
    } catch {
      uiStore.showToast("Failed to load system status");
    } finally {
      loading.value = false;
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
    uiStore.showToast("Reboot failed");
  }
}

export const sysStore = {
  get device() {
    return device.value;
  },
  get version() {
    return version.value;
  },
  get systemInfo() {
    return systemInfo.value;
  },
  get loading() {
    return loading.value;
  },
  get hasLoaded() {
    return hasLoaded;
  },
  ensureStatusLoaded,
  loadStatus,
  rebootDevice,
};
