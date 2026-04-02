/**
 * Copyright 2025 Magic Mount-rs Authors
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import type { AppAPI, AppConfig, Module, StorageStatus } from "../types";
import { MockAPI } from "./api.mock";
import { DEFAULT_CONFIG, PATHS } from "./constants";

interface KsuExecResult {
  errno: number;
  stdout: string;
  stderr: string;
}

type KsuExec = (cmd: string) => Promise<KsuExecResult>;

let ksuExec: KsuExec | null = null;

try {
  const ksu = await import("kernelsu").catch(() => null);

  ksuExec = ksu ? ksu.exec : null;
} catch {}

const shouldUseMock = import.meta.env.DEV || !ksuExec;

const shellEscapeDoubleQuoted = (value: string): string =>
  value.replace(/(["\\$`])/g, "\\$1");

function stringToHex(str: string): string {
  const bytes = new TextEncoder().encode(str);
  let hex = "";

  for (const byte of bytes) {
    hex += byte.toString(16).padStart(2, "0");
  }

  return hex;
}

function normalizeConfigPayload(payload: Record<string, unknown>): AppConfig {
  const ignoreListSource = Array.isArray(payload.ignoreList)
    ? payload.ignoreList
    : Array.isArray(payload.ignore_list)
      ? payload.ignore_list
      : [];
  const disableUmount =
    typeof payload.disable_umount === "boolean"
      ? payload.disable_umount
      : undefined;

  return {
    ...DEFAULT_CONFIG,
    mountsource:
      typeof payload.mountsource === "string"
        ? payload.mountsource
        : DEFAULT_CONFIG.mountsource,
    partitions: Array.isArray(payload.partitions)
      ? payload.partitions.filter((value): value is string => !!value)
      : [],
    ignoreList: ignoreListSource.filter(
      (value): value is string => typeof value === "string" && value.length > 0,
    ),
    umount:
      typeof payload.umount === "boolean"
        ? payload.umount
        : disableUmount === undefined
          ? DEFAULT_CONFIG.umount
          : !disableUmount,
  };
}

const createStandardConfigPayload = (config: AppConfig) => ({
  mountsource: config.mountsource,
  partitions: config.partitions,
  ignoreList: config.ignoreList,
  disable_umount: !config.umount,
});

function normalizeModule(module: Record<string, unknown>): Module {
  const skipMount =
    typeof module.skipMount === "boolean"
      ? module.skipMount
      : typeof module.skip === "boolean"
        ? module.skip
        : false;
  const mode =
    typeof module.mode === "string"
      ? module.mode
      : skipMount
        ? "ignore"
        : "magic";
  const rawRules =
    module.rules &&
    typeof module.rules === "object" &&
    !Array.isArray(module.rules)
      ? (module.rules as Record<string, unknown>)
      : {};
  const rawPaths =
    rawRules.paths &&
    typeof rawRules.paths === "object" &&
    !Array.isArray(rawRules.paths)
      ? (rawRules.paths as Record<string, unknown>)
      : {};

  return {
    id: String(module.id ?? ""),
    name: String(module.name ?? module.id ?? "Unknown"),
    version: String(module.version ?? ""),
    author: String(module.author ?? "Unknown"),
    description: String(module.description ?? ""),
    is_mounted:
      typeof module.is_mounted === "boolean" ? module.is_mounted : !skipMount,
    mode,
    disabledByFlag:
      typeof module.disabledByFlag === "boolean"
        ? module.disabledByFlag
        : undefined,
    skipMount,
    rules: {
      default_mode:
        typeof rawRules.default_mode === "string"
          ? rawRules.default_mode
          : mode,
      paths: rawPaths,
    },
  };
}

function formatBytes(bytes: number, decimals = 2): string {
  if (!+bytes) {
    return "0 B";
  }

  const k = 1024;
  const dm = decimals < 0 ? 0 : decimals;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));

  return `${Number.parseFloat((bytes / k ** i).toFixed(dm))} ${sizes[i]}`;
}

const RealAPI: AppAPI = {
  loadConfig: async () => {
    const { errno, stdout, stderr } = await ksuExec!(
      `${PATHS.BINARY} show-config`,
    );

    if (errno === 0 && stdout.trim()) {
      return normalizeConfigPayload(JSON.parse(stdout));
    }

    throw new Error(stderr || "show-config failed");
  },

  saveConfig: async (config) => {
    const payload = stringToHex(
      JSON.stringify(createStandardConfigPayload(config)),
    );
    const { errno, stderr } = await ksuExec!(
      `${PATHS.BINARY} save-config --payload ${payload}`,
    );

    if (errno !== 0) {
      throw new Error(stderr || "save-config failed");
    }
  },

  scanModules: async () => {
    const { errno, stdout, stderr } = await ksuExec!(`${PATHS.BINARY} modules`);

    if (errno === 0 && stdout) {
      return JSON.parse(stdout).map((module: Record<string, unknown>) =>
        normalizeModule(module),
      );
    }

    throw new Error(stderr || "modules failed");
  },

  getStorageUsage: async () => {
    let type: StorageStatus["type"] = null;

    try {
      const { errno, stdout } = await ksuExec!(`cat "${PATHS.DAEMON_STATE}"`);

      if (errno === 0 && stdout) {
        const state = JSON.parse(stdout);

        if (typeof state.storage_mode === "string") {
          type = state.storage_mode;
        }
      }
    } catch {}

    try {
      const { stdout } = await ksuExec!(
        `df -k "${PATHS.MODULE_ROOT}" | tail -n 1`,
      );

      if (stdout) {
        const parts = stdout.split(/\s+/);

        if (parts.length >= 6) {
          const total = Number.parseInt(parts[1]) * 1024;
          const used = Number.parseInt(parts[2]) * 1024;
          const percent = parts[4];

          return {
            type: type ?? "ext4",
            percent,
            size: formatBytes(total),
            used: formatBytes(used),
          };
        }
      }
    } catch {}

    return {
      size: "-",
      used: "-",
      percent: "0%",
      type,
    };
  },

  getSystemInfo: async () => {
    try {
      const cmd = `
        echo "KERNEL:$(uname -r)"
        echo "SELINUX:$(getenforce)"
      `;
      const { errno, stdout } = await ksuExec!(cmd);
      const info = {
        kernel: "-",
        selinux: "-",
        mountBase: PATHS.MODULE_ROOT,
        activeMounts: [] as string[],
      };

      if (errno === 0 && stdout) {
        for (const line of stdout.split("\n")) {
          if (line.startsWith("KERNEL:")) {
            info.kernel = line.slice(7).trim();
          } else if (line.startsWith("SELINUX:")) {
            info.selinux = line.slice(8).trim();
          }
        }
      }

      try {
        const { errno: stateErrno, stdout: stateStdout } = await ksuExec!(
          `cat "${PATHS.DAEMON_STATE}"`,
        );

        if (stateErrno === 0 && stateStdout) {
          const state = JSON.parse(stateStdout);

          if (typeof state.mount_point === "string") {
            info.mountBase = state.mount_point;
          }
          if (Array.isArray(state.active_mounts)) {
            info.activeMounts = state.active_mounts.filter(
              (value: unknown): value is string => typeof value === "string",
            );
          }
        }
      } catch {}

      if (info.activeMounts.length === 0) {
        const modulesResult = await ksuExec!(`ls -1 "${PATHS.MODULE_ROOT}"`);

        if (modulesResult.errno === 0 && modulesResult.stdout) {
          info.activeMounts = modulesResult.stdout
            .split("\n")
            .filter((s) => s.trim() && s !== "magic_mount_rs");
        }
      }

      return info;
    } catch {
      return { kernel: "-", selinux: "-", mountBase: "-", activeMounts: [] };
    }
  },

  getDeviceStatus: async () => {
    const cmd = `
      getprop ro.product.model
      getprop ro.build.version.release
      getprop ro.build.version.sdk
    `;
    const { stdout } = await ksuExec!(cmd);
    const lines = stdout ? stdout.split("\n") : [];
    const androidVersion = lines[1]?.trim() || "Unknown";
    const apiLevel = lines[2]?.trim();

    return {
      model: lines[0]?.trim() || "Unknown",
      android: apiLevel
        ? `${androidVersion} (API ${apiLevel})`
        : androidVersion,
      kernel: "See System Info",
      selinux: "See System Info",
    };
  },

  getVersion: async () => {
    const cmd = `${PATHS.BINARY} version`;

    try {
      const { errno, stdout } = await ksuExec!(cmd);

      if (errno === 0 && stdout) {
        try {
          const res = JSON.parse(stdout);

          return res.version ?? "0.0.0";
        } catch {
          return stdout.trim() || "0.0.0";
        }
      }
    } catch {}

    return "Unknown";
  },

  openLink: async (url) => {
    const safeUrl = shellEscapeDoubleQuoted(url);
    const cmd = `am start -a android.intent.action.VIEW -d "${safeUrl}"`;

    await ksuExec!(cmd);
  },

  reboot: async () => {
    const cmd = "svc power reboot || reboot";

    await ksuExec!(cmd);
  },
};

export const API: AppAPI = shouldUseMock ? MockAPI : RealAPI;
