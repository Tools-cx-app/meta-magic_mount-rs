import { createRoot, createSignal } from "solid-js";

import type { DeviceInfo, StorageStatus, SystemInfo } from "../../types";
import { API } from "../api";
import { uiStore } from "./uiStore";

function createSysStore() {
  const [device, setDevice] = createSignal<DeviceInfo>({
    model: "-",
    android: "-",
    kernel: "-",
    selinux: "-",
  });
  const [version, setVersion] = createSignal("...");
  const [storage, setStorage] = createSignal<StorageStatus>({
    used: "-",
    size: "-",
    percent: "0%",
    type: null,
  });
  const [systemInfo, setSystemInfo] = createSignal<SystemInfo>({
    kernel: "-",
    selinux: "-",
    mountBase: "-",
    activeMounts: [],
  });
  const [activePartitions, setActivePartitions] = createSignal<string[]>([]);
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
        const [baseDevice, nextVersion, nextStorage, info] = await Promise.all([
          API.getDeviceStatus(),
          API.getVersion(),
          API.getStorageUsage(),
          API.getSystemInfo(),
        ]);

        setDevice({
          ...baseDevice,
          kernel: info.kernel,
          selinux: info.selinux,
        });
        setVersion(nextVersion);
        setStorage(nextStorage);
        setSystemInfo(info);
        setActivePartitions(info.activeMounts || []);
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
    get storage() {
      return storage();
    },
    get systemInfo() {
      return systemInfo();
    },
    get activePartitions() {
      return activePartitions();
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
