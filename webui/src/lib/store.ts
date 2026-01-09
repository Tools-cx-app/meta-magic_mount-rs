/**
 * Copyright 2025 Magic Mount-rs Authors
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import { createMemo, createRoot, createSignal } from "solid-js";

import type {
  DeviceStatus,
  MagicConfig,
  MagicModule,
  StorageUsage,
  SystemInfo,
} from "../types";
import { API } from "./api";
import { DEFAULT_CONFIG, DEFAULT_SEED } from "./constants";
import { availableLanguages, locales } from "./i18n";
import { Monet } from "./theme";

let darkModeQuery: MediaQueryList;

function createStore() {
  const [theme, setThemeSignal] = createSignal("auto");
  const [isSystemDark, setIsSystemDark] = createSignal(false);
  const [lang, setLangSignal] = createSignal("en");
  const [seed, setSeed] = createSignal(DEFAULT_SEED);
  const [toast, setToast] = createSignal({
    id: "init",
    text: "",
    type: "info",
    visible: false,
  });

  const [fixBottomNav, setFixBottomNav] = createSignal(false);

  const [config, setConfig] = createSignal<MagicConfig>(DEFAULT_CONFIG);
  const [modules, setModules] = createSignal<MagicModule[]>([]);

  const [device, setDevice] = createSignal<DeviceStatus>({
    model: "-",
    android: "-",
    kernel: "-",
    selinux: "-",
  });
  const [version, setVersion] = createSignal("...");
  const [storage, setStorage] = createSignal<StorageUsage>({
    used: "-",
    size: "-",
    percent: "0%",
    type: null,
    hymofs_available: false,
  });
  const [systemInfo, setSystemInfo] = createSignal<SystemInfo>({
    kernel: "-",
    selinux: "-",
    mountBase: "-",
    activeMounts: [],
  });
  const [activePartitions, setActivePartitions] = createSignal<string[]>([]);
  const [diagnostics, setDiagnostics] = createSignal<any[]>([]);

  const [loadingConfig, setLoadingConfig] = createSignal(false);
  const [loadingModules, setLoadingModules] = createSignal(false);
  const [loadingStatus, setLoadingStatus] = createSignal(false);
  const [loadingDiagnostics, setLoadingDiagnostics] = createSignal(false);

  const [savingConfig, setSavingConfig] = createSignal(false);
  const [savingModules] = createSignal(false);

  const L = createMemo(() => locales[lang()] ?? locales.en);

  const modeStats = createMemo(() => {
    const stats = { auto: 0, magic: 0, hymofs: 0 };
    for (const m of modules()) {
      if (!m.is_mounted) {
        continue;
      }
      stats.magic++;
    }

    return stats;
  });

  const toasts = createMemo(() => {
    const t = toast();

    return t.visible ? [t] : [];
  });

  function showToast(text: string, type = "info") {
    const id = Date.now().toString();
    setToast({ id, text, type, visible: true });
    setTimeout(() => {
      if (toast().id === id) {
        setToast((prev) => ({ ...prev, visible: false }));
      }
    }, 3000);
  }

  function setTheme(t: string) {
    setThemeSignal(t);
    localStorage.setItem("mm-theme", t);
    applyTheme();
  }

  function toggleBottomNavFix() {
    setFixBottomNav((prev) => {
      const newValue = !prev;
      localStorage.setItem("mm-fix-nav", String(newValue));

      return newValue;
    });
  }

  function applyTheme() {
    const isDark = theme() === "auto" ? isSystemDark() : theme() === "dark";
    document.documentElement.setAttribute(
      "data-theme",
      isDark ? "dark" : "light",
    );
    Monet.apply(seed(), isDark);
  }

  function setLang(code: string) {
    setLangSignal(code);
    localStorage.setItem("mm-lang", code);
  }

  async function init() {
    const savedLang = localStorage.getItem("mm-lang") ?? "en";
    setLangSignal(savedLang);

    const savedTheme = localStorage.getItem("mm-theme");
    if (savedTheme) {
      setThemeSignal(savedTheme);
    }

    const savedNavFix = localStorage.getItem("mm-fix-nav");
    if (savedNavFix === "true") {
      setFixBottomNav(true);
    }

    if (!darkModeQuery && typeof window !== "undefined") {
      darkModeQuery = window.matchMedia("(prefers-color-scheme: dark)");
      setIsSystemDark(darkModeQuery.matches);
      darkModeQuery.addEventListener("change", (e) => {
        setIsSystemDark(e.matches);
        applyTheme();
      });
    }

    try {
      const sysColor = await API.fetchSystemColor();
      if (sysColor) {
        setSeed(sysColor);
      }
    } catch {}

    applyTheme();

    await Promise.all([loadConfig(), loadStatus()]);
  }

  async function loadConfig() {
    setLoadingConfig(true);
    try {
      setConfig(await API.loadConfig());
    } catch {
      showToast("Failed to load config", "error");
    }
    setLoadingConfig(false);
  }

  async function resetConfig() {
    setLoadingConfig(true);
    try {
      setConfig({ ...DEFAULT_CONFIG });
      await saveConfig();
      showToast(L().common.resetSuccess, "success");
    } catch {
      showToast(L().common.resetFailed, "error");
    }
    setLoadingConfig(false);
  }

  async function saveConfig() {
    setSavingConfig(true);
    try {
      await API.saveConfig(config());
      showToast(L().config.saveSuccess, "success");
    } catch {
      showToast("Failed to save config", "error");
    }
    setSavingConfig(false);
  }

  async function loadModules() {
    setLoadingModules(true);
    try {
      setModules(await API.scanModules());
    } catch {
      showToast("Failed to load modules", "error");
    }
    setLoadingModules(false);
  }

  async function saveModules() {
    showToast("Not supported in this version", "info");
  }

  async function loadStatus() {
    setLoadingStatus(true);
    setLoadingDiagnostics(true);
    try {
      const baseDevice = await API.getDeviceStatus();
      setVersion(await API.getVersion());
      setStorage(await API.getStorageUsage());
      const sysInfo = await API.getSystemInfo();
      setSystemInfo(sysInfo);
      setActivePartitions(sysInfo.activeMounts || []);
      setDiagnostics([]);

      setDevice({
        ...baseDevice,
        kernel: sysInfo.kernel,
        selinux: sysInfo.selinux,
      });

      if (modules().length === 0) {
        await loadModules();
      }
    } catch {}
    setLoadingStatus(false);
    setLoadingDiagnostics(false);
  }

  async function rebootDevice() {
    try {
      await API.reboot();
    } catch {
      showToast(L().common.rebootFailed, "error");
    }
  }

  return {
    get theme() {
      return theme();
    },
    get isSystemDark() {
      return isSystemDark();
    },
    get lang() {
      return lang();
    },
    get seed() {
      return seed();
    },
    get availableLanguages() {
      return availableLanguages;
    },
    get L() {
      return L();
    },
    get toast() {
      return toast();
    },
    get toasts() {
      return toasts();
    },
    showToast,
    setTheme,
    setLang,
    init,

    get fixBottomNav() {
      return fixBottomNav();
    },
    toggleBottomNavFix,

    get config() {
      return config();
    },
    setConfig,
    loadConfig,
    saveConfig,
    resetConfig,

    get modules() {
      return modules();
    },
    setModules,
    get modeStats() {
      return modeStats();
    },
    loadModules,
    saveModules,

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
    get diagnostics() {
      return diagnostics();
    },
    loadStatus,
    rebootDevice,

    get loading() {
      return {
        config: loadingConfig(),
        modules: loadingModules(),
        status: loadingStatus(),
        diagnostics: loadingDiagnostics(),
      };
    },
    get saving() {
      return {
        config: savingConfig(),
        modules: savingModules(),
      };
    },
  };
}

export const store = createRoot(createStore);
