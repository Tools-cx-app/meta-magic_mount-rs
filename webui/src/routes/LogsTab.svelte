<script>
  import { store } from '../lib/store.svelte';
  import { ICONS } from '../lib/constants';
  import { onMount, tick, onDestroy } from 'svelte';
  import Skeleton from '../components/Skeleton.svelte';
  import './LogsTab.css';
  let searchLogQuery = $state('');
  let filterLevel = $state('all'); 
  let logContainer;
  let autoRefresh = $state(false);
  let refreshInterval;
  let filteredLogs = $derived(store.logs.filter(line => {
    const text = line.text.toLowerCase();
    const matchesSearch = text.includes(searchLogQuery.toLowerCase());
    let matchesLevel = true;
    if (filterLevel !== 'all') {
      matchesLevel = line.type === filterLevel;
    }
    return matchesSearch && matchesLevel;
  }));
  async function scrollToBottom() {
    if (logContainer && autoRefresh) { 
      await tick();
      logContainer.scrollTop = logContainer.scrollHeight;
    }
  }
  async function refreshLogs(silent = false) {
    await store.loadLogs(silent);
    if (!silent) scrollToBottom(); 
  }
  async function copyLogs() {
    if (filteredLogs.length === 0) return;
    const text = filteredLogs.map(l => l.text).join('\n');
    try {
      await navigator.clipboard.writeText(text);
      store.showToast(store.L.logs.copySuccess, 'success');
    } catch (e) {
      store.showToast(store.L.logs.copyFail, 'error');
    }
  }
  $effect(() => {
    if (autoRefresh) {
      refreshLogs(true); 
      refreshInterval = setInterval(() => {
        refreshLogs(true); 
      }, 3000);
    } else {
      if (refreshInterval) clearInterval(refreshInterval);
    }
    return () => { if (refreshInterval) clearInterval(refreshInterval); };
  });
  onMount(() => {
    refreshLogs(); 
  });
  onDestroy(() => {
    if (refreshInterval) clearInterval(refreshInterval);
  });
</script>
<div class="logs-controls">
  <svg viewBox="0 0 24 24" width="20" height="20" style="fill: var(--md-sys-color-on-surface-variant)">
    <path d={ICONS.search} />
  </svg>
  <input 
    type="text" 
    class="log-search-input" 
    placeholder={store.L.logs.searchPlaceholder}
    bind:value={searchLogQuery}
  />
  <div style="display:flex; align-items:center; gap:6px; margin-right:8px;">
    <input type="checkbox" id="auto-refresh" bind:checked={autoRefresh} style="accent-color: var(--md-sys-color-primary);" />
    <label for="auto-refresh" style="font-size: 12px; color: var(--md-sys-color-on-surface-variant); cursor: pointer; white-space: nowrap;">Auto</label>
  </div>
  <div style="height: 16px; width: 1px; background: var(--md-sys-color-outline-variant); margin: 0 8px;"></div>
  <span style="font-size: 12px; color: var(--md-sys-color-on-surface-variant); white-space: nowrap;">
    {store.L.logs.filterLabel}
  </span>
  <select class="log-filter-select" bind:value={filterLevel}>
    <option value="all">{store.L.logs.levels.all}</option>
    <option value="info">{store.L.logs.levels.info}</option>
    <option value="warn">{store.L.logs.levels.warn}</option>
    <option value="error">{store.L.logs.levels.error}</option>
  </select>
</div>
<div class="log-container" bind:this={logContainer}>
  {#if store.loading.logs}
    <div style="display:flex; flex-direction:column; gap:8px;">
      {#each Array(10) as _, i}
        <Skeleton width="{60 + (i % 3) * 20}%" height="14px" />
      {/each}
    </div>
  {:else if filteredLogs.length === 0}
    <div style="padding: 20px; text-align: center;">
      {store.logs.length === 0 ? store.L.logs.empty : "No matching logs"}
    </div>
  {:else}
    {#each filteredLogs as line}
      <span class="log-entry">
        <span class="log-{line.type}">{line.text}</span>
      </span>
    {/each}
    <div style="text-align: center; padding: 12px; font-size: 11px; opacity: 0.5; border-top: 1px solid rgba(255,255,255,0.1); margin-top: 12px;">
      — Showing last 1000 lines —
    </div>
  {/if}
</div>
<div class="bottom-actions">
  <button class="btn-tonal" onclick={copyLogs} disabled={filteredLogs.length === 0} title={store.L.logs.copy}>
    <svg viewBox="0 0 24 24" width="20" height="20"><path d={ICONS.copy} fill="currentColor"/></svg>
  </button>
  <div style="flex:1"></div>
  <button 
    class="btn-tonal" 
    onclick={() => refreshLogs(false)} 
    disabled={store.loading.logs}
    title={store.L.logs.refresh}
  >
    <svg viewBox="0 0 24 24" width="20" height="20"><path d={ICONS.refresh} fill="currentColor"/></svg>
  </button>
</div>