export interface AppConfig {
  mountsource: string;
  umount: boolean;
  partitions: string[];
  ignoreList: string[];
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
