import { DEFAULT_CONFIG, PATHS } from './constants';
import { MockAPI } from './api.mock';

let ksuExec;
try {
  const ksu = await import('kernelsu').catch(() => null);
  ksuExec = ksu ? ksu.exec : null;
} catch (e) {
  console.warn("KernelSU module not found, defaulting to Mock/Fallback.");
}

const shouldUseMock = import.meta.env.DEV || !ksuExec;

console.log(`[API Init] Mode: ${shouldUseMock ? '🛠️ MOCK (Dev/Browser)' : '🚀 REAL (Device)'}`);

const RealAPI = {
  loadConfig: async () => {
    const cmd = `${PATHS.BINARY} show-config`;
    try {
      const { errno, stdout } = await ksuExec(cmd);
      if (errno === 0 && stdout) {
        return JSON.parse(stdout);
      } else {
        console.warn("Config load returned non-zero or empty, using defaults");
        return DEFAULT_CONFIG;
      }
    } catch (e) {
      console.error("Failed to load config from backend:", e);
      return DEFAULT_CONFIG; 
    }
  },

  saveConfig: async (config) => {
    const jsonStr = JSON.stringify(config);
    
    let bytes;
    if (typeof TextEncoder !== 'undefined') {
      const encoder = new TextEncoder();
      bytes = encoder.encode(jsonStr);
    } else {
      bytes = new Uint8Array(jsonStr.length);
      for (let i = 0; i < jsonStr.length; i++) {
        bytes[i] = jsonStr.charCodeAt(i) & 0xFF;
      }
    }
    
    let hexPayload = '';
    for (let i = 0; i < bytes.length; i++) {
      const hex = bytes[i].toString(16);
      hexPayload += (hex.length === 1 ? '0' + hex : hex);
    }

    const cmd = `${PATHS.BINARY} save-config --payload ${hexPayload}`;
    const { errno, stderr } = await ksuExec(cmd);
    
    if (errno !== 0) {
      throw new Error(`Failed to save config: ${stderr}`);
    }
  },

  scanModules: async () => {
    const cmd = `${PATHS.BINARY} modules`;
    try {
      const { errno, stdout } = await ksuExec(cmd);
      if (errno === 0 && stdout) {
        return JSON.parse(stdout);
      }
    } catch (e) {
      console.error("Module scan failed:", e);
    }
    return [];
  },

  saveModules: async (modules) => {
    let content = "# Module Modes\n";
    modules.forEach(m => { 
      if (m.mode !== 'auto' && /^[a-zA-Z0-9_.-]+$/.test(m.id)) {
        content += `${m.id}=${m.mode}\n`; 
      }
    });
    
    const data = content.replace(/'/g, "'\\''");
    const modeConfigPath = PATHS.MODE_CONFIG || "/data/adb/meta-hybrid/module_mode.conf";
    const cmd = `mkdir -p "$(dirname "${modeConfigPath}")" && printf '%s\n' '${data}' > "${modeConfigPath}"`;
    
    const { errno } = await ksuExec(cmd);
    if (errno !== 0) throw new Error('Failed to save modes');
  },

  readLogs: async (logPath, lines = 1000) => {
    const f = logPath || DEFAULT_CONFIG.logfile;
    const cmd = `[ -f "${f}" ] && tail -n ${lines} "${f}" || echo ""`;
    const { errno, stdout, stderr } = await ksuExec(cmd);
    
    if (errno === 0) return stdout || "";
    throw new Error(stderr || "Log file not found or unreadable");
  },

  getStorageUsage: async () => {
    try {
      const cmd = `${PATHS.BINARY} storage`;
      const { errno, stdout } = await ksuExec(cmd);
      
      if (errno === 0 && stdout) {
        return JSON.parse(stdout);
      }
    } catch (e) {
      console.error("Storage check failed:", e);
    }
    return { size: '-', used: '-', percent: '0%', type: null };
  },

  getSystemInfo: async () => {
    try {
      const cmdSys = `echo "KERNEL:$(uname -r)"; echo "SELINUX:$(getenforce)"`;
      const { errno: errSys, stdout: outSys } = await ksuExec(cmdSys);
      
      let info = { kernel: '-', selinux: '-', mountBase: '-' };
      if (errSys === 0 && outSys) {
        outSys.split('\n').forEach(line => {
          if (line.startsWith('KERNEL:')) info.kernel = line.substring(7).trim();
          else if (line.startsWith('SELINUX:')) info.selinux = line.substring(8).trim();
        });
      }

      const stateFile = PATHS.DAEMON_STATE || "/data/adb/meta-hybrid/run/daemon_state.json";
      const cmdState = `cat "${stateFile}"`;
      const { errno: errState, stdout: outState } = await ksuExec(cmdState);
      
      if (errState === 0 && outState) {
        try {
          const state = JSON.parse(outState);
          info.mountBase = state.mount_point || 'Unknown';
        } catch (e) {
          console.error("Failed to parse daemon state JSON", e);
        }
      }

      return info;
    } catch (e) {
      console.error("System info check failed:", e);
      return { kernel: 'Unknown', selinux: 'Unknown', mountBase: 'Unknown' };
    }
  },

  getActiveMounts: async (sourceName) => {
    try {
      const src = sourceName || DEFAULT_CONFIG.mountsource;
      const cmd = `mount | grep "${src}"`; 
      const { errno, stdout } = await ksuExec(cmd);
      
      const mountedParts = [];
      if (errno === 0 && stdout) {
        stdout.split('\n').forEach(line => {
          const parts = line.split(' ');
          if (parts.length >= 3 && parts[2].startsWith('/')) {
            const partName = parts[2].substring(1);
            if (partName) mountedParts.push(partName);
          }
        });
      }
      return mountedParts;
    } catch (e) {
      console.error("Mount check failed:", e);
      return [];
    }
  },

  openLink: async (url) => {
    const safeUrl = url.replace(/"/g, '\\"');
    const cmd = `am start -a android.intent.action.VIEW -d "${safeUrl}"`;
    await ksuExec(cmd);
  },

  fetchSystemColor: async () => {
    try {
      const { stdout } = await ksuExec('settings get secure theme_customization_overlay_packages');
      if (stdout) {
        const match = /["']?android\.theme\.customization\.system_palette["']?\s*:\s*["']?#?([0-9a-fA-F]{6,8})["']?/i.exec(stdout) || 
                      /["']?source_color["']?\s*:\s*["']?#?([0-9a-fA-F]{6,8})["']?/i.exec(stdout);
        if (match && match[1]) {
          let hex = match[1];
          if (hex.length === 8) hex = hex.substring(2);
          return '#' + hex;
        }
      }
    } catch (e) {}
    return null;
  }
};

export const API = shouldUseMock ? MockAPI : RealAPI;