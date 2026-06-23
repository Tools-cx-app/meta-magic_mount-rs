/*
 * Copyright (C) 2026 meta-magic_mount-rs developers
 * SPDX-License-Identifier: GPL-v3
 */

export interface CustomMount {
  source: string;
  target: string;
}

export interface AppConfig {
  mountsource: string;
  umount: boolean;
  partitions: string[];
  ignoreList: string[];
  customMounts: CustomMount[];
}

export interface Module {
  id: string;
  name: string;
  version: string;
  author: string;
  description: string;
  isMounted: boolean;
  bottomopen?: boolean;
}

export interface SystemInfo {
  kernel: string | null;
  selinux: string | null;
}

export interface DeviceInfo {
  model: string | null;
}

export interface Status {
  version: string;
  device: DeviceInfo;
  system: SystemInfo;
}

export interface ConnectionInfo {
  port: number;
  token: string;
}

export interface ErrorResponse {
  error: {
    code:
      | "invalidRequest"
      | "invalidConfig"
      | "unauthorized"
      | "notFound"
      | "conflict"
      | "internal"
      | "unavailable";
    message: string;
  };
}

export interface LanguageOption {
  code: string;
  name: string;
}

export interface AppAPI {
  loadConfig: () => Promise<AppConfig>;
  saveConfig: (config: AppConfig) => Promise<void>;
  scanModules: () => Promise<Module[]>;
  getSystemInfo: () => Promise<SystemInfo>;
  getDeviceStatus: () => Promise<DeviceInfo>;
  getVersion: () => Promise<string>;
  openLink: (url: string) => Promise<void>;
  reboot: () => Promise<void>;
}
