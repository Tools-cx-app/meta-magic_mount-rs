export interface MagicConfig {
  tempdir?: string;
  mountsource: string;
  umount: boolean;
  partitions: string[];
}

export interface MagicModule {
  id: string;
  name: string;
  version: string;
  author: string;
  description: string;
  is_mounted: boolean;
  mode: string;
  disabledByFlag?: boolean;
  skipMount?: boolean;
  rules: { default_mode: string; paths: Record<string, any> };
}

export interface SystemInfo {
  kernel: string;
  selinux: string;
  mountBase: string;
  activeMounts: string[];
}

export interface StorageUsage {
  type: string | null;
  percent: string;
  size: string;
  used: string;
  hymofs_available: boolean;
}

export interface DeviceStatus {
  model: string;
  android: string;
  kernel: string;
  selinux: string;
}

export interface APIType {
  loadConfig: () => Promise<MagicConfig>;
  saveConfig: (config: MagicConfig) => Promise<void>;
  scanModules: () => Promise<MagicModule[]>;
  getStorageUsage: () => Promise<StorageUsage>;
  getSystemInfo: () => Promise<SystemInfo>;
  getDeviceStatus: () => Promise<DeviceStatus>;
  getVersion: () => Promise<string>;
  openLink: (url: string) => Promise<void>;
  fetchSystemColor: () => Promise<string | null>;
  reboot: () => Promise<void>;
}
