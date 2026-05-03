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
  is_mounted: boolean;
}

export interface SystemInfo {
  kernel: string;
  selinux: string;
}

export interface DeviceInfo {
  model: string;
}

export type ToastType = "info" | "success" | "error";

export interface ToastMessage {
  id: string;
  text: string;
  type: ToastType;
  visible: boolean;
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
