<script>
  import { onMount } from 'svelte';
  import { store } from '../lib/store.svelte';
  import { ICONS } from '../lib/constants';
  import './InfoTab.css';
  import Skeleton from '../components/Skeleton.svelte';

  // Adapted for the current project
  const REPO_OWNER = 'YuzakiKokuban';
  const REPO_NAME = 'meta-hybrid_mount';
  // Sponsor link placeholder (using GitHub Sponsors if available, or just project page)
  const DONATE_LINK = `https://github.com/sponsors/${REPO_OWNER}`; 

  let contributors = $state([]);
  let loading = $state(true);
  let error = $state(false);

  onMount(async () => {
    await fetchContributors();
  });

  async function fetchContributors() {
    try {
      // 1. Fetch basic contributor list
      const res = await fetch(`https://api.github.com/repos/${REPO_OWNER}/${REPO_NAME}/contributors`);
      if (!res.ok) throw new Error('Failed to fetch list');
      
      const basicList = await res.json();
      
      // 2. Fetch detailed user info to get bio (GitHub API rate limit applies)
      // Since there might be many contributors, we process them in parallel.
      const detailPromises = basicList.map(async (user) => {
        try {
            // Skip detailed fetch for bots
            if (user.type === 'Bot') return { ...user, bio: 'Bot' };

            const detailRes = await fetch(user.url); // user.url is the API url for user details
            if (detailRes.ok) {
                const detail = await detailRes.json();
                return { ...user, bio: detail.bio, name: detail.name || user.login };
            }
        } catch (e) {
            console.warn('Failed to fetch detail for', user.login);
        }
        return user; // Fallback to basic info on failure
      });

      contributors = await Promise.all(detailPromises);
    } catch (e) {
      console.error(e);
      error = true;
    } finally {
      loading = false;
    }
  }
</script>

<div class="info-container">
  
  <div class="project-header">
    <div class="app-logo">
        <svg viewBox="0 0 24 24" width="32" height="32"><path d={ICONS.home} fill="currentColor"/></svg>
    </div>
    <span class="app-name">{store.L.common.appName}</span>
    <span class="app-version">{store.version}</span>
  </div>

  <div class="action-grid">
    <a href={`https://github.com/${REPO_OWNER}/${REPO_NAME}`} target="_blank" class="action-card">
        <svg viewBox="0 0 24 24" class="action-icon"><path d={ICONS.github} /></svg>
        <span class="action-label">{store.L.info.projectLink}</span>
    </a>
    <a href={DONATE_LINK} target="_blank" class="action-card">
        <svg viewBox="0 0 24 24" class="action-icon" style="fill: #ffab91;"><path d={ICONS.donate} /></svg>
        <span class="action-label">{store.L.info.donate}</span>
    </a>
  </div>

  <div>
    <div class="section-title">{store.L.info.contributors}</div>
    
    <div class="contributors-list">
        {#if loading}
            {#each Array(3) as _}
                <div class="contributor-bar">
                    <Skeleton width="48px" height="48px" borderRadius="50%" />
                    <div class="c-info">
                        <Skeleton width="120px" height="16px" style="margin-bottom: 6px;" />
                        <Skeleton width="200px" height="12px" />
                    </div>
                </div>
            {/each}
        {:else if error}
            <div style="text-align:center; padding: 20px; opacity: 0.6; color: var(--md-sys-color-error);">
                {store.L.info.loadFail}
            </div>
        {:else}
            {#each contributors as user}
                <a href={user.html_url} target="_blank" class="contributor-bar">
                    <img src={user.avatar_url} alt={user.login} class="c-avatar" />
                    <div class="c-info">
                        <span class="c-name">{user.name || user.login}</span>
                        <span class="c-bio">
                            {user.bio || store.L.info.noBio}
                        </span>
                    </div>
                    <svg viewBox="0 0 24 24" class="c-link-icon"><path d={ICONS.share} /></svg>
                </a>
            {/each}
        {/if}
    </div>
  </div>

</div>