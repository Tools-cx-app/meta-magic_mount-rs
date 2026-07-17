<!--

    Copyright (C) 2026 meta-magic_mount-rs developers
    SPDX-License-Identifier: GPL-v3

-->
<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import Skeleton from "../components/md3/Skeleton.vue";
import MagicLogo from "../components/md3/logo.vue";
import { ICONS } from "../lib/constants";
import { sysStore } from "../lib/stores/sysStore";
import axios from "axios";
import { API } from "../lib/api";

const REPO_OWNER = "Tools-cx-app";
const REPO_NAME = "meta-magic_mount-rs";

interface Contributor {
  id: number;
  login: string;
  name: string;
  url: string;
  html_url: string;
  bio: string | null;
  type: string;
  avatar_url: string;
}

const version = ref("");

const { t } = useI18n();
const contributors = ref<Contributor[]>([]);
const loading = ref(true);
const error = ref(false);

API.getVersion().then((ver) => {
  version.value = ver;
});

const STORAGE_KEY = "mmrs_contributors";

const cached = localStorage.getItem(STORAGE_KEY);
if (cached) {
  try {
    const parsed = JSON.parse(cached);
    if (
      parsed.timestamp &&
      Date.now() - parsed.timestamp < 24 * 60 * 60 * 1000
    ) {
      contributors.value = parsed.data;
      loading.value = false;
    } else {
      localStorage.removeItem(STORAGE_KEY);
    }
  } catch {
    localStorage.removeItem(STORAGE_KEY);
  }
}

if (!contributors.value.length) {
  axios
    .get<Contributor[]>(
      "https://api.github.com/repos/Tools-cx-app/meta-magic_mount-rs/contributors",
    )
    .then(function (response) {
      const userContributors = response.data.filter(function (contributor) {
        return contributor.type === "User";
      });
      const detailPromises = userContributors.map(function (contributor) {
        return axios
          .get(contributor.url)
          .then(function (detailResponse) {
            return {
              ...contributor,
              bio: detailResponse.data.bio ?? null,
              name: detailResponse.data.name ?? contributor.login,
            };
          })
          .catch(function () {
            return {
              ...contributor,
              bio: null,
              name: contributor.login,
            };
          });
      });
      return Promise.all(detailPromises);
    })
    .then(function (result) {
      contributors.value = result.map((item) => ({
        ...item,
        bio: item.bio ?? null,
      }));
      localStorage.setItem(
        STORAGE_KEY,
        JSON.stringify({
          timestamp: Date.now(),
          data: result,
        }),
      );
      loading.value = false;
    })
    .catch(function () {
      error.value = true;
      loading.value = false;
    });
}

function handleLink(event: MouseEvent, url: string) {
  event.preventDefault();
  void API.openLink(url);
}
</script>

<template>
  <div class="md3-page info-container">
    <div class="project-header">
      <div class="app-logo">
        <MagicLogo />
      </div>
      <span class="app-name">{{ t("common.appName") }}</span>
      <span class="app-version">{{ sysStore.version }}</span>
    </div>

    <div class="action-buttons">
      <button
        class="action-btn action-btn-wide"
        @click="
          (event: MouseEvent) =>
            handleLink(event, `https://github.com/${REPO_OWNER}/${REPO_NAME}`)
        "
      >
        <svg class="btn-icon" viewBox="0 0 24 24" width="20" height="20">
          <path :d="ICONS.github" />
        </svg>
        <span>{{ t("info.projectLink") }}</span>
      </button>
    </div>

    <div class="contributors-section">
      <div class="section-title">{{ t("info.contributors") }}</div>

      <div class="list-wrapper">
        <template v-if="loading">
          <div v-for="i in 3" :key="i" class="skeleton-item">
            <Skeleton class="skeleton-avatar" />
            <div class="skeleton-text">
              <Skeleton class="skeleton-text-title" />
              <Skeleton class="skeleton-text-body" />
            </div>
          </div>
        </template>

        <template v-else>
          <template v-if="!error">
            <div class="contributors-list">
              <div
                v-for="user in contributors"
                :key="user.login"
                class="contributor-item"
                @click="(event: MouseEvent) => handleLink(event, user.html_url)"
              >
                <img
                  :src="`${user.avatar_url}${user.avatar_url.includes('?') ? '&' : '?'}s=80`"
                  :alt="user.login"
                  class="c-avatar"
                  loading="lazy"
                />
                <div class="contributor-info">
                  <div class="contributor-name">
                    {{ user.name ?? user.login }}
                  </div>
                  <div class="contributor-bio">
                    {{ user.bio ?? t("info.noBio") }}
                  </div>
                </div>
              </div>
            </div>
          </template>

          <template v-else>
            <div class="error-message">{{ t("info.loadFail") }}</div>
          </template>
        </template>
      </div>
    </div>
  </div>
</template>

<style scoped>
.info-container {
  display: flex;
  flex-direction: column;
  gap: 24px;
  padding-bottom: 32px;
}

.project-header {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 16px;
  padding: 32px 0 28px;
  background-color: var(--md-sys-color-surface-container);
  border-radius: 32px;
  user-select: none;
}

.app-logo {
  width: 152px;
  height: 180px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.app-logo svg {
  width: 100%;
  height: 100%;
  overflow: visible;
}

.app-name {
  font-size: 32px;
  font-weight: 600;
  color: var(--md-sys-color-on-surface);
  margin-top: 0;
  letter-spacing: -0.5px;
}

.app-version {
  font-family: var(--md-ref-typeface-mono);
  font-size: 14px;
  background: var(--md-sys-color-surface-container-high);
  padding: 8px 20px;
  border-radius: 9999px;
  color: var(--md-sys-color-primary);
  font-weight: 500;
}

.action-buttons {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

.action-btn {
  width: 100%;
  height: 56px;
  border-radius: 16px;
  border: none;
  background-color: var(--md-sys-color-secondary-container);
  color: var(--md-sys-color-on-secondary-container);
  font-size: 14px;
  font-weight: 500;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  cursor: pointer;
  transition: background-color 0.2s;
}

.action-btn:hover {
  background-color: var(--md-sys-color-secondary-container-high);
}

.action-btn-wide {
  grid-column: 1 / -1;
}

.btn-icon {
  fill: currentColor;
}

.contributors-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.section-title {
  font-size: 14px;
  font-weight: 500;
  margin: 0 var(--md-sys-shape-corner-large);
  color: var(--md-sys-color-primary);
  user-select: none;
}

.list-wrapper {
  display: flex;
  flex-direction: column;
  gap: 2px;
  border-radius: var(--md-sys-shape-corner-large);
  overflow: clip;
}

.contributors-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.contributor-item {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 12px 16px;
  background-color: var(--md-sys-color-surface-container);
  cursor: pointer;
  transition: background-color 0.2s;
}

.contributor-item:hover {
  background-color: var(--md-sys-color-surface-container-high);
}

.c-avatar {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  background-color: var(--md-sys-color-surface-variant);
  object-fit: cover;
  flex-shrink: 0;
}

.contributor-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.contributor-name {
  font-size: 14px;
  font-weight: 500;
  color: var(--md-sys-color-on-surface);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.contributor-bio {
  font-size: 12px;
  color: var(--md-sys-color-on-surface-variant);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.skeleton-item {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 12px 16px;
}

.skeleton-text {
  display: flex;
  flex-direction: column;
  gap: 6px;
  flex: 1;
}

.error-message {
  text-align: center;
  padding: 24px;
  opacity: 0.7;
  background: var(--md-sys-color-error-container);
  color: var(--md-sys-color-on-error-container);
  border-radius: 24px;
  margin: 0 12px;
}

@media (max-width: 640px) {
  .project-header {
    padding: 28px 0 24px;
  }

  .app-logo {
    width: 132px;
    height: 158px;
  }
}
</style>
