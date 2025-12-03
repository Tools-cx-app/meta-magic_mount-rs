const delay = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

export const MockAPI = {
  loadConfig: async () => {
    await delay(300);
    console.log('[Mock] loadConfig called');
    return {
      moduledir: '/data/adb/modules',
      tempdir: '',
      mountsource: 'KSU',
      logfile: '/data/adb/meta-hybrid/daemon.log',
      verbose: true,
      partitions: ['my_product', 'mi_ext'],
      force_ext4: false,
      enable_nuke: true,
      disable_umount: false
    };
  },

  saveConfig: async (config) => {
    await delay(500);
    console.log('[Mock] saveConfig received:', config);
  },

  scanModules: async () => {
    await delay(800);
    console.log('[Mock] scanModules called');
    return [
      {
        id: "youtube-revanced",
        name: "YouTube ReVanced",
        version: "v19.11.43",
        author: "ReVanced Team",
        description: "Extended YouTube functionality.",
        mode: "auto"
      },
      {
        id: "miui-home-mod",
        name: "MIUI Home Mod",
        version: "v4.25.0",
        author: "Unknown",
        description: "Modify system launcher.",
        mode: "magic"
      }
    ];
  },

  saveModules: async (modules) => {
    await delay(500);
    console.log('[Mock] saveModules received:', modules);
  },

  readLogs: async (logPath, lines) => {
    await delay(400);
    return `[INFO] >> Initializing Meta-Hybrid Mount Daemon...
[INFO] >> Storage Backend: [TMPFS]
[INFO] >> Inventory Scan: Found 2 enabled modules.
[INFO] [OverlayFS Fusion Sequence]
[INFO] ├── [Target] /vendor
[INFO] │   ╰── [Layer] miui-home-mod
[INFO] >> Link Start! Executing mount plan...
[INFO] >> System operational.`;
  },

  getStorageUsage: async () => {
    await delay(600);
    return {
      size: '2.0G',
      used: '350M',
      avail: '1.65G',
      percent: '17%',
      type: 'ext4'
    };
  },

  getSystemInfo: async () => {
    await delay(300);
    return {
      kernel: '6.6.77-android15-Kokuban-Herta-BYIF-SukiSUU-gc4abb096d-4k',
      selinux: 'Enforcing',
      mountBase: '/data/adb/meta-hybrid/img_mnt'
    };
  },

  getActiveMounts: async (sourceName) => {
    await delay(400);
    return ['system', 'vendor'];
  },

  openLink: async (url) => {
    console.log('[Mock] Opening URL:', url);
    window.open(url, '_blank');
  },

  fetchSystemColor: async () => {
    await delay(200);
    console.log('[Mock] fetchSystemColor returning #D0BCFF');
    return '#bcffff'; 
  }
};