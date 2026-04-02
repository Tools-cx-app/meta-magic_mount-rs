export type MountMode = "overlay" | "magic" | "ignore";

export interface ModuleRules {
  default_mode: MountMode | string;
  paths: Record<string, unknown>;
}

export interface AppConfig {
  mountsource: string;
  umount: boolean;
  partitions: string[];
  ignoreList: string[];
}

export type MagicConfig = AppConfig;

export interface Module {
  id: string;
  name: string;
  version: string;
  author: string;
  description: string;
  is_mounted: boolean;
  mode: MountMode | string;
  disabledByFlag?: boolean;
  skipMount?: boolean;
  rules: ModuleRules;
}

export type MagicModule = Module;

export interface SystemInfo {
  kernel: string;
  selinux: string;
  mountBase: string;
  activeMounts: string[];
}

export interface StorageStatus {
  type: string | null;
  percent: string;
  size: string;
  used: string;
}

export type StorageUsage = StorageStatus;

export interface DeviceInfo {
  model: string;
  android: string;
  kernel: string;
  selinux: string;
}

export type DeviceStatus = DeviceInfo;

export interface AppAPI {
  loadConfig: () => Promise<AppConfig>;
  saveConfig: (config: AppConfig) => Promise<void>;
  scanModules: () => Promise<Module[]>;
  getStorageUsage: () => Promise<StorageStatus>;
  getSystemInfo: () => Promise<SystemInfo>;
  getDeviceStatus: () => Promise<DeviceInfo>;
  getVersion: () => Promise<string>;
  openLink: (url: string) => Promise<void>;
  reboot: () => Promise<void>;
}

export interface ToastMessage {
  id: string;
  text: string;
  type: "info" | "success" | "error";
  visible: boolean;
}

export interface LanguageOption {
  code: string;
  name: string;
}

export interface ModeStats {
  overlay: number;
  magic: number;
}

export type APIType = AppAPI;
