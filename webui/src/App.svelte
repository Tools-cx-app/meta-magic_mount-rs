<script>
  import { onMount } from 'svelte';
  import { exec } from 'kernelsu';
  import locate from './locate.json';
  import './app.css';

  const DEFAULT_CONFIG = {
    moduledir: '/data/adb/modules',
    tempdir: '',
    mountsource: 'HybridMount',
    logfile: '/data/adb/meta-hybrid/daemon.log',
    verbose: false,
    partitions: []
  };

  const CONFIG_PATH = '/data/adb/meta-hybrid/config.toml';
  const MODE_CONFIG_PATH = '/data/adb/meta-hybrid/module_mode.conf';
  const IMAGE_MNT_PATH = '/data/adb/meta-hybrid/mnt';

  const icons = {
    settings: "M19.14 12.94c.04-.3.06-.61.06-.94 0-.32-.02-.64-.07-.94l2.03-1.58a.49.49 0 0 0 .12-.61l-1.92-3.32a.488.488 0 0 0-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54a.484.484 0 0 0-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.56-1.62.94l-2.39-.96c-.22-.08-.47 0-.59.22L2.74 8.87c-.12.21-.08.47.12.61l2.03 1.58c-.05.3-.09.63-.09.94s.02.64.07.94l-2.03 1.58a.49.49 0 0 0-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32c.12-.22.07-.47-.12-.61l-2.03-1.58zM12 15.6c-1.98 0-3.6-1.62-3.6-3.6s1.62-3.6 3.6-3.6 3.6 1.62 3.6 3.6-1.62 3.6-3.6 3.6z",
    modules: "M2 4v16h20V4H2zm18 14H4V6h16v12zM6 10h12v2H6v-2zm0 4h8v2H6v-2z", 
    description: "M14 2H6c-1.1 0-1.99.9-1.99 2L4 20c0 1.1.89 2 1.99 2H18c1.1 0 2-.9 2-2V8l-6-6zm2 16H8v-2h8v2zm0-4H8v-2h8v2zm-3-5V3.5L18.5 9H13z",
    save: "M17 3H5c-1.11 0-2 .9-2 2v14c0 1.1.89 2 2 2h14c1.1 0 2-.9 2-2V7l-4-4zm-5 16c-1.66 0-3-1.34-3-3s1.34-3 3-3 3 1.34 3 3-1.34 3-3 3zm3-10H5V5h10v4z",
    refresh: "M17.65 6.35C16.2 4.9 14.21 4 12 4c-4.42 0-7.99 3.58-7.99 8s3.57 8 7.99 8c3.73 0 6.84-2.55 7.73-6h-2.08c-.82 2.33-3.04 4-5.65 4-3.31 0-6-2.69-6-6s2.69-6 6-6c1.66 0 3.14.69 4.22 1.78L13 11h7V4l-2.35 2.35z",
    translate: "M12.87 15.07l-2.54-2.51.03-.03A17.52 17.52 0 0 0 14.07 6H17V4h-7V2H8v2H1v2h11.17C11.5 7.92 10.44 9.75 9 11.35 8.07 10.32 7.3 9.19 6.69 8h-2c.73 1.63 1.73 3.17 2.98 4.56l-5.09 5.02L4 19l5-5 3.11 3.11.76-2.04zM18.5 10h-2L12 22h2l1.12-3h4.75L21 22h2l-4.5-12zm-2.62 7l1.62-4.33L19.12 17h-3.24z",
    light_mode: "M12 7c-2.76 0-5 2.24-5 5s2.24 5 5 5 5-2.24 5-5-2.24-5-5-5zM2 13h2c.55 0 1-.45 1-1s-.45-1-1-1H2c-.55 0-1 .45-1 1s.45 1 1 1zm18 0h2c.55 0 1-.45 1-1s-.45-1-1-1h-2c-.55 0-1 .45-1 1s.45 1 1 1zM11 2v2c0 .55.45 1 1 1s1-.45 1-1V2c0-.55-.45-1-1-1s-1 .45-1 1zm0 18v2c0 .55.45 1 1 1s1-.45 1-1v-2c0-.55-.45-1-1-1s-1 .45-1 1z",
    dark_mode: "M12 3c-4.97 0-9 4.03-9 9s4.03 9 9 9 9-4.03 9-9c0-.46-.04-.92-.1-1.36-.98 1.37-2.58 2.26-4.4 2.26-2.98 0-5.4-2.42-5.4-5.4 0-1.81.89-3.42 2.26-4.4-.44-.06-.9-.1-1.36-.1z",
    extension: "M20.5 11H19V7c0-1.1-.9-2-2-2h-4V3.5C13 2.12 11.88 1 10.5 1S8 2.12 8 3.5V5H4c-1.1 0-1.99.9-1.99 2v3.8H3.5c1.38 0 2.5 1.12 2.5 2.5s-1.12 2.5-2.5 2.5H2v2c0 1.1.9 2 2 2h3.8c0-1.38 1.12-2.5 2.5-2.5s2.5 1.12 2.5 2.5H17c1.1 0 2-.9 2-2v-4h1.5c1.38 0 2.5-1.12 2.5-2.5S21.88 11 20.5 11z"
  };

  let lang = 'en';
  let theme = 'dark';
  $: L = locate[lang];

  const availableLanguages = Object.keys(locate).map(code => ({
    code,
    name: locate[code]?.lang?.display || code.toUpperCase()
  }));

  let showLangMenu = false;
  let activeTab = 'config';
  const TABS = [
    { id: 'config', icon: icons.settings },
    { id: 'modules', icon: icons.modules },
    { id: 'logs', icon: icons.description }
  ];

  let config = { ...DEFAULT_CONFIG };
  let partitionInput = '';
  let modules = [];
  let loading = { config: false, modules: false, logs: false };
  let saving = { config: false, modules: false };
  let messages = { config: null, modules: null, logs: null };

  let logContent = '';
  let logSelection = 'current';

  function parseKvConfig(text) {
    try {
      const result = { ...DEFAULT_CONFIG };
      text.split('\n').forEach(line => {
        line = line.trim();
        if (!line || line.startsWith('#')) return;
        const eq = line.indexOf('=');
        if (eq < 0) return;
        let key = line.slice(0, eq).trim();
        let val = line.slice(eq + 1).trim();
        if (val.startsWith('"') && val.endsWith('"')) val = val.slice(1, -1);
        
        if (key === 'moduledir') result.moduledir = val;
        else if (key === 'tempdir') result.tempdir = val;
        else if (key === 'mountsource') result.mountsource = val;
        else if (key === 'verbose') result.verbose = val === 'true';
        else if (key === 'partitions') result.partitions = val.split(',').map(s => s.trim()).filter(Boolean);
      });
      return result;
    } catch (e) { return null; }
  }

  function serializeKvConfig(cfg) {
    const q = s => `"${s}"`;
    const lines = ['# Hybrid Mount Config', ''];
    lines.push(`moduledir=${q(cfg.moduledir)}`);
    if (cfg.tempdir) lines.push(`tempdir=${q(cfg.tempdir)}`);
    lines.push(`mountsource=${q(cfg.mountsource)}`);
    lines.push(`verbose=${cfg.verbose}`);
    if (cfg.partitions.length) lines.push(`partitions=${q(cfg.partitions.join(','))}`);
    return lines.join('\n');
  }

  async function loadConfig() {
    loading.config = true;
    try {
      const { errno, stdout } = await exec(`[ -f "${CONFIG_PATH}" ] && cat "${CONFIG_PATH}" || echo ""`);
      config = (errno === 0 && stdout.trim()) ? (parseKvConfig(stdout) || DEFAULT_CONFIG) : DEFAULT_CONFIG;
      partitionInput = config.partitions.join(', ');
      messages.config = L.config.loadSuccess;
    } catch (e) { messages.config = L.config.loadError; }
    loading.config = false;
  }

  async function saveConfig() {
    saving.config = true;
    try {
      config.partitions = partitionInput.split(',').map(s => s.trim()).filter(Boolean);
      const data = serializeKvConfig(config).replace(/'/g, "'\\''");
      const cmd = `mkdir -p "$(dirname "${CONFIG_PATH}")" && printf '%s\n' '${data}' > "${CONFIG_PATH}"`;
      const { errno } = await exec(cmd);
      messages.config = errno === 0 ? L.config.saveSuccess : L.config.saveFailed;
    } catch (e) { messages.config = L.config.saveFailed; }
    saving.config = false;
  }

  async function loadModules() {
    loading.modules = true;
    modules = [];
    try {
      const { stdout: modeOut } = await exec(`[ -f "${MODE_CONFIG_PATH}" ] && cat "${MODE_CONFIG_PATH}" || echo ""`);
      const modeMap = new Map();
      modeOut.split('\n').forEach(l => {
        const [id, m] = l.split('=').map(s => s.trim());
        if (id) modeMap.set(id, m);
      });

      const dir = config.moduledir || DEFAULT_CONFIG.moduledir;
      const imgDir = IMAGE_MNT_PATH;
      
      // Modified Shell Command (Improved Logic):
      // 1. Check if module is enabled (not disabled/skipped/removed).
      // 2. Check for content in EITHER the standard module dir OR the hybrid image mount dir.
      // 3. Only list if it has content in at least one of those locations.
      const cmd = `
        cd "${dir}" && for d in *; do
          if [ -d "$d" ] && \
             [ ! -f "$d/disable" ] && \
             [ ! -f "$d/skip_mount" ] && \
             [ ! -f "$d/remove" ]; then
             
             HAS_CONTENT=false
             
             # Check Standard Directory
             if [ -d "$d/system" ] || [ -d "$d/vendor" ] || [ -d "$d/product" ] || [ -d "$d/system_ext" ] || [ -d "$d/odm" ] || [ -d "$d/oem" ]; then
               HAS_CONTENT=true
             fi
             
             # Check Hybrid Image Directory (if standard check failed)
             if [ "$HAS_CONTENT" = "false" ]; then
                if [ -d "${imgDir}/$d/system" ] || [ -d "${imgDir}/$d/vendor" ] || [ -d "${imgDir}/$d/product" ] || [ -d "${imgDir}/$d/system_ext" ] || [ -d "${imgDir}/$d/odm" ] || [ -d "${imgDir}/$d/oem" ]; then
                  HAS_CONTENT=true
                fi
             fi
             
             if [ "$HAS_CONTENT" = "true" ]; then
               echo "$d"
             fi
          fi
        done
      `;
      
      const { errno, stdout } = await exec(cmd);

      if (errno === 0) {
        modules = stdout.split('\n')
          .map(s => s.trim())
          .filter(s => s && !['meta-hybrid', 'meta-overlayfs', 'magic_mount'].includes(s))
          .map(id => ({ id, mode: modeMap.get(id) || 'auto' }));
      } else {
        messages.modules = L.modules.scanError;
      }
    } catch (e) { messages.modules = L.modules.scanError; }
    loading.modules = false;
  }

  async function saveModules() {
    saving.modules = true;
    try {
      let content = "# Module Modes\n";
      modules.forEach(m => { if (m.mode !== 'auto') content += `${m.id}=${m.mode}\n`; });
      const data = content.replace(/'/g, "'\\''");
      const { errno } = await exec(`mkdir -p "$(dirname "${MODE_CONFIG_PATH}")" && printf '%s\n' '${data}' > "${MODE_CONFIG_PATH}"`);
      messages.modules = errno === 0 ? L.modules.saveSuccess : L.modules.saveFailed;
    } catch (e) { messages.modules = L.modules.saveFailed; }
    saving.modules = false;
  }

  async function loadLog() {
    loading.logs = true;
    logContent = '';
    try {
      const f = logSelection === 'current' ?
        (config.logfile || DEFAULT_CONFIG.logfile) : `${config.logfile || DEFAULT_CONFIG.logfile}.old`;
      const { errno, stdout, stderr } = await exec(`[ -f "${f}" ] && cat "${f}" || echo "Log empty"`);
      logContent = errno === 0 ? stdout : `${L.logs.readFailed}: ${stderr}`;
    } catch (e) { messages.logs = L.logs.readException; }
    loading.logs = false;
  }

  onMount(() => {
    if (typeof window !== 'undefined') {
      const savedLang = window.localStorage.getItem('mm-lang');
      if (savedLang && locate[savedLang]) lang = savedLang;
      const savedTheme = window.localStorage.getItem('mm-theme');
      setTheme(savedTheme || (window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'));
    }
    loadConfig();
  });

  function setTheme(t) {
    theme = t;
    if (typeof document !== 'undefined') {
      document.documentElement.setAttribute('data-theme', t);
      window.localStorage.setItem('mm-theme', t);
    }
  }

  function toggleTheme() { 
    setTheme(theme === 'light' ? 'dark' : 'light');
  }
  
  $: if (activeTab === 'modules') loadModules();
  $: if (activeTab === 'logs') loadLog();
</script>

<div class="app-root">
  <header class="app-bar">
    <div class="app-bar-content">
      <h1 class="screen-title">{L.tabs[activeTab]}</h1>
      <div class="top-actions">
        <button class="btn-icon" on:click={toggleTheme}><svg viewBox="0 0 24 24" width="24" height="24"><path d={theme === 'light' ? icons.dark_mode : icons.light_mode} fill="currentColor"/></svg></button>
        <button class="btn-icon" on:click={() => showLangMenu = !showLangMenu}><svg viewBox="0 0 24 24" width="24" height="24"><path d={icons.translate} fill="currentColor"/></svg></button>
        {#if showLangMenu}
          <div class="menu-dropdown">
            {#each availableLanguages as l}
              <button class="menu-item" on:click={() => { lang = l.code; showLangMenu = false; window.localStorage.setItem('mm-lang', l.code); }} class:selected={lang === l.code}>{l.name}</button>
            {/each}
          </div>
        {/if}
      </div>
    </div>
  </header>

  <nav class="nav-container">
    {#each TABS as tab}
      <button class="nav-item {activeTab === tab.id ? 'active' : ''}" on:click={() => activeTab = tab.id}>
        <span class="icon-box"><svg viewBox="0 0 24 24" class="nav-icon"><path d={tab.icon} fill="currentColor"/></svg></span>
        <span class="label">{L.tabs[tab.id]}</span>
      </button>
    {/each}
  </nav>

  <main class="main-content fade-in">
    <div class="container-max">
      {#if activeTab === 'config'}
        <div class="md3-card">
          <div class="switch-wrap"><span>{L.config.verboseLabel}</span><label class="md3-switch"><input type="checkbox" bind:checked={config.verbose}><span class="track"><span class="thumb"></span></span></label></div>
          <div class="text-field" style="margin-top: 16px;"><input type="text" id="c-moduledir" bind:value={config.moduledir} placeholder={DEFAULT_CONFIG.moduledir} /><label for="c-moduledir">{L.config.moduleDir}</label></div>
          <div class="text-field"><input type="text" id="c-tempdir" bind:value={config.tempdir} placeholder="Auto" /><label for="c-tempdir">{L.config.tempDir}</label></div>
          <div class="text-field"><input type="text" id="c-mountsource" bind:value={config.mountsource} placeholder={DEFAULT_CONFIG.mountsource} /><label for="c-mountsource">{L.config.mountSource}</label></div>
          <div class="text-field"><input type="text" id="c-partitions" bind:value={partitionInput} placeholder="mi_ext, my_stock" /><label for="c-partitions">{L.config.partitions}</label></div>
          <div class="actions-row">
             <button class="btn-text" on:click={loadConfig} disabled={loading.config}>{L.config.reload}</button>
             <button class="btn-filled" on:click={saveConfig} disabled={saving.config}><svg viewBox="0 0 24 24" width="18" height="18"><path d={icons.save} fill="currentColor"/></svg>{saving.config ? '...' : L.config.save}</button>
          </div>
          {#if messages.config}<div class="msg-toast">{messages.config}</div>{/if}
        </div>
      {:else if activeTab === 'modules'}
        <div class="actions-bar">
          <p class="desc-text" style="margin:0; flex:1; font-size:14px; opacity:0.8">{L.modules.desc}</p>
          <button class="btn-tonal" on:click={loadModules} disabled={loading.modules} title={L.modules.reload}><svg viewBox="0 0 24 24" width="18" height="18"><path d={icons.refresh} fill="currentColor"/></svg></button>
          <button class="btn-filled" on:click={saveModules} disabled={saving.modules}><svg viewBox="0 0 24 24" width="18" height="18"><path d={icons.save} fill="currentColor"/></svg>{saving.modules ? '...' : L.modules.save}</button>
        </div>
        {#if messages.modules}<div class="msg-toast">{messages.modules}</div>{/if}
        <div class="rules-list">
          {#if modules.length === 0}
            <div style="text-align:center; padding: 20px; opacity: 0.7">{loading.modules ? 'Loading...' : L.modules.empty}</div>
          {:else}
            {#each modules as mod (mod.id)}
              <div class="rule-card">
                <div style="flex: 2; display:flex; align-items:center; gap:12px; min-width: 180px;">
                  <div class="icon-box" style="background:var(--md-sys-color-surface-variant); width:40px; height:40px; border-radius:8px;">
                      <svg viewBox="0 0 24 24" width="24" height="24"><path d={icons.extension} fill="currentColor"/></svg>
                  </div>
                  <span style="font-weight:500;">{mod.id}</span>
                </div>
                <div class="text-field mode-select" style="margin-bottom:0 !important">
                   <select bind:value={mod.mode}>
                    <option value="auto">{L.modules.modeAuto}</option>
                    <option value="magic">{L.modules.modeMagic}</option>
                  </select>
                </div>
              </div>
            {/each}
          {/if}
        </div>
      {:else if activeTab === 'logs'}
        <div class="actions-bar logs-actions">
          <div class="text-field" style="margin-bottom:0; width: 100%; max-width: 200px;">
            <select bind:value={logSelection} on:change={loadLog}>
              <option value="current">{L.logs.current}</option>
              <option value="old">{L.logs.old}</option>
            </select>
          </div>
          <button class="btn-tonal" on:click={loadLog} disabled={loading.logs}>
            <svg viewBox="0 0 24 24" width="18" height="18"><path d={icons.refresh} fill="currentColor"/></svg>
            {L.logs.refresh}
          </button>
        </div>
        <div class="log-container">{loading.logs ? 'Loading...' : (logContent || L.logs.empty)}</div>
      {/if}
    </div>
  </main>
</div>