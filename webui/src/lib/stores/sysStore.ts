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
