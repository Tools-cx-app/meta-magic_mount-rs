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

import type { AppAPI, AppConfig, CustomMount, Module } from "../types";
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

  const customMountsSource = Array.isArray(payload.customMounts)
    ? payload.customMounts
    : Array.isArray(payload.custom_mounts)
      ? payload.custom_mounts
      : [];

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
    customMounts: customMountsSource.filter(
      (value): value is CustomMount =>
        typeof value === "object" &&
        value !== null &&
        typeof value.source === "string" &&
        value.source.length > 0 &&
        typeof value.target === "string" &&
        value.target.length > 0,
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
  customMounts: config.customMounts
    .map((mount) => ({
      source: mount.source.trim(),
      target: mount.target.trim(),
    }))
    .filter((mount) => mount.source.length > 0 && mount.target.length > 0),
  disable_umount: !config.umount,
});

function normalizeModule(module: Record<string, unknown>): Module {
  const skipMount =
    typeof module.skipMount === "boolean"
      ? module.skipMount
      : typeof module.skip === "boolean"
        ? module.skip
        : false;

  return {
    id: String(module.id ?? ""),
    name: String(module.name ?? module.id ?? "Unknown"),
    version: String(module.version ?? ""),
    author: String(module.author ?? "Unknown"),
    description: String(module.description ?? ""),
    is_mounted:
      typeof module.is_mounted === "boolean" ? module.is_mounted : !skipMount,
  };
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

  saveConfig: async (config: AppConfig) => {
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

      return info;
    } catch {
      return { kernel: "-", selinux: "-" };
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

    return {
      model: lines[0]?.trim() || "Unknown",
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

  openLink: async (url: string) => {
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
