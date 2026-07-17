<!--

    Copyright (C) 2026 meta-magic_mount-rs developers
    SPDX-License-Identifier: GPL-v3

-->
<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { useI18n } from "vue-i18n";
import BottomActions from "../components/md3/BottomActions.vue";
import Skeleton from "../components/md3/Skeleton.vue";
import Searchbar from "../components/md3/searchbar.vue";
import { ICONS } from "../lib/constants";
import { moduleStore } from "../lib/stores/moduleStore";

const { t } = useI18n();

const searchQuery = ref("");
const expandedId = ref<string | null>(null);

const filteredModules = computed(() => {
  if (searchQuery.value.trim() === "") {
    return moduleStore.modules;
  }
  const query = searchQuery.value.toLowerCase();
  return moduleStore.modules.filter(
    (module) =>
      module.name.toLowerCase().includes(query) ||
      module.description.toLowerCase().includes(query) ||
      module.id.toLowerCase().includes(query),
  );
});

onMounted(async () => {
  await moduleStore.loadModules();

  moduleStore.modules.forEach((module) => {
    module.bottomopen = false;
  });
});

function toggleExpand(id: string) {
  expandedId.value = expandedId.value === id ? null : id;
}

function getModeLabel(isMounted: boolean) {
  return isMounted ? "MOUNTED" : "UNMOUNTED";
}
</script>

<template>
  <div class="md3-page modules-page">
    <div class="header-section">
      <Searchbar v-model="searchQuery" />
    </div>

    <div class="modules-list">
      <template v-if="moduleStore.loading">
        <Skeleton v-for="i in 6" :key="i" class="skeleton-module-card" />
      </template>

      <template v-else>
        <template v-if="filteredModules.length > 0">
          <div
            v-for="module in filteredModules"
            :key="module.id"
            class="module-card"
            :class="{
              expanded: expandedId === module.id,
              unmounted: !module.is_mounted,
            }"
          >
            <div
              class="module-header"
              @click="toggleExpand(module.id)"
              role="button"
              tabindex="0"
              @keydown="
                (e) =>
                  (e.key === 'Enter' || e.key === ' ') &&
                  toggleExpand(module.id)
              "
            >
              <div class="module-info">
                <div class="module-name">{{ module.name }}</div>
                <div class="module-meta">
                  <span class="module-id">{{ module.id }}</span>
                  <span class="version-badge">{{ module.version }}</span>
                </div>
              </div>
              <div
                class="mode-indicator"
                :class="{
                  'mode-mounted': module.is_mounted,
                  'mode-unmounted': !module.is_mounted,
                }"
              >
                {{ getModeLabel(module.is_mounted) }}
              </div>
            </div>

            <div class="module-body-wrapper">
              <div class="module-body-inner">
                <div class="module-body-content">
                  <div class="body-section">
                    <div class="section-label">
                      {{ t("modules.descriptionLabel") }}
                    </div>
                    <p class="module-desc">
                      {{
                        module.description ?? t("modules.noDescriptionLabel")
                      }}
                    </p>
                  </div>

                  <div class="body-section">
                    <div class="section-label">
                      {{ t("modules.authorLabel") }}
                    </div>
                    <div class="module-author">
                      {{ module.author ?? t("modules.unknownLabel") }}
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </template>

        <template v-else>
          <div class="empty-state">
            <div class="empty-icon">
              <svg viewBox="0 0 24 24" width="48" height="48">
                <path :d="ICONS.modules" />
              </svg>
            </div>
            <p>
              {{
                moduleStore.modules.length === 0
                  ? t("modules.empty")
                  : t("modules.emptyState")
              }}
            </p>
          </div>
        </template>
      </template>
    </div>

    <BottomActions>
      <div class="spacer" />
      <button
        class="action-btn-icon"
        @click="moduleStore.loadModules()"
        :title="t('modules.reload')"
      >
        <svg viewBox="0 0 24 24" width="24" height="24">
          <path :d="ICONS.refresh" />
        </svg>
      </button>
    </BottomActions>
  </div>
</template>

<style scoped>
.modules-page {
  display: flex;
  flex-direction: column;
  gap: 12px;
  max-width: 800px;
  width: 100%;
  margin: 0 auto;
}

.header-section {
  display: flex;
  flex-direction: column;
  gap: 12px;
  position: sticky;
  top: 0;
  z-index: 50;
  background: linear-gradient(
    180deg,
    var(--md-sys-color-background) 85%,
    transparent
  );
}

.modules-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
  border-radius: var(--md-sys-shape-corner-large);
  overflow: clip;
}

.module-card {
  background-color: var(--md-sys-color-surface-container);
  border-radius: 4px;
  transition: all 0.3s cubic-bezier(0.2, 0, 0, 1);
  position: relative;
}

.module-card:active {
  border-radius: var(--md-sys-shape-corner-large);
}

.module-card.expanded {
  background-color: var(--md-sys-color-surface-container-high);
}

.module-card.unmounted {
  opacity: 0.75;
}

.module-header {
  padding: 12px 16px;
  display: flex;
  align-items: center;
  gap: 12px;
  cursor: pointer;
  min-height: 64px;
}

.module-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.module-name {
  font-size: 14.5px;
  font-weight: 500;
  color: var(--md-sys-color-on-surface);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: 1.3;
}

.module-meta {
  font-size: 11px;
  color: var(--md-sys-color-on-surface-variant);
  display: flex;
  gap: 6px;
  align-items: center;
  white-space: nowrap;
  overflow: hidden;
}

.module-id {
  overflow: hidden;
  text-overflow: ellipsis;
  opacity: 0.9;
}

.version-badge {
  background-color: var(--md-sys-color-surface-variant);
  padding: 0 5px;
  border-radius: 4px;
  font-family: var(--md-ref-typeface-mono);
  flex-shrink: 0;
  height: 16px;
  line-height: 16px;
}

.mode-indicator {
  padding: 4px 8px;
  border-radius: 6px;
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  flex-shrink: 0;
  margin-left: 4px;
  letter-spacing: 0.3px;
}

.mode-mounted {
  background-color: var(--md-sys-color-primary-container);
  color: var(--md-sys-color-on-primary-container);
}

.mode-unmounted {
  background-color: var(--md-sys-color-surface-variant);
  color: var(--md-sys-color-on-surface-variant);
}

.module-body-wrapper {
  display: grid;
  grid-template-rows: 0fr;
  transition: grid-template-rows 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.module-card.expanded .module-body-wrapper {
  grid-template-rows: 1fr;
}

.module-body-inner {
  overflow: hidden;
}

.module-body-content {
  padding: 0 16px 16px 16px;
  border-top: 1px solid var(--md-sys-color-outline-variant);
  opacity: 0;
  transform: translateY(-8px);
  transition:
    opacity 0.2s ease,
    transform 0.2s ease;
  transition-delay: 0s;
}

.module-card.expanded .module-body-content {
  opacity: 1;
  transform: translateY(0);
  transition-delay: 0.1s;
}

.body-section {
  margin-top: 12px;
}

.section-label {
  font-size: 11px;
  font-weight: 500;
  color: var(--md-sys-color-primary);
  text-transform: uppercase;
  margin-bottom: 4px;
  letter-spacing: 0.5px;
}

.module-desc {
  font-size: 13px;
  color: var(--md-sys-color-on-surface);
  line-height: 1.45;
  margin: 0;
}

.module-author {
  font-size: 13px;
  color: var(--md-sys-color-on-surface);
}

.empty-state {
  text-align: center;
  padding: 48px 0;
  color: var(--md-sys-color-outline);
  font-size: 14px;
}

.empty-icon {
  fill: var(--md-sys-color-outline);
  margin-bottom: 16px;
  opacity: 0.5;
}

.spacer {
  flex: 1;
}

.action-btn-icon {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  border: none;
  background-color: var(--md-sys-color-secondary-container);
  color: var(--md-sys-color-on-secondary-container);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: background-color 0.2s;
}

.action-btn-icon svg {
  fill: currentColor;
}
</style>
