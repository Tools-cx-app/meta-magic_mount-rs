<!--

    Copyright (C) 2026 meta-magic_mount-rs developers
    SPDX-License-Identifier: GPL-v3

-->
<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import Skeleton from "../components/md3/Skeleton.vue";
import BottomActions from "../components/md3/BottomActions.vue";
import { ICONS } from "../lib/constants";
import { configStore } from "../lib/stores/configStore";
import { moduleStore } from "../lib/stores/moduleStore";
import { sysStore } from "../lib/stores/sysStore";
import { API } from "../lib/api";

const { t } = useI18n();

const showRebootConfirm = ref(false);

const mountedCount = computed(
  () => moduleStore.modules.filter((module) => module.is_mounted).length,
);

const moduleStatsReady = computed(
  () => !moduleStore.loading && moduleStore.hasLoaded,
);

function reboot() {
  showRebootConfirm.value = false;
  void API.reboot();
}

onMounted(async () => {
  await sysStore.ensureStatusLoaded();
  await moduleStore.ensureModulesLoaded();
});
</script>

<template>
  <div class="md3-page">
    <div class="dashboard-grid">
      <div class="hero-card">
        <div class="hero-bg-decoration">
          <svg class="hero-corner-blossom" viewBox="0 0 120 120">
            <defs>
              <path
                id="hero-blossom-petal-shape"
                d="M60 8C73 8 84 20 84 34C84 52 71 64 64 72C62 74 61 77 60 82C59 77 58 74 56 72C49 64 36 52 36 34C36 20 47 8 60 8Z"
              />
              <mask id="hero-blossom-outline-mask">
                <rect width="120" height="120" fill="white" />
                <g fill="black">
                  <use href="#hero-blossom-petal-shape" />
                  <use
                    href="#hero-blossom-petal-shape"
                    transform="rotate(72 60 60)"
                  />
                  <use
                    href="#hero-blossom-petal-shape"
                    transform="rotate(144 60 60)"
                  />
                  <use
                    href="#hero-blossom-petal-shape"
                    transform="rotate(216 60 60)"
                  />
                  <use
                    href="#hero-blossom-petal-shape"
                    transform="rotate(288 60 60)"
                  />
                </g>
              </mask>
            </defs>
            <g class="hero-blossom-petals">
              <use href="#hero-blossom-petal-shape" />
              <use
                href="#hero-blossom-petal-shape"
                transform="rotate(72 60 60)"
              />
              <use
                href="#hero-blossom-petal-shape"
                transform="rotate(144 60 60)"
              />
              <use
                href="#hero-blossom-petal-shape"
                transform="rotate(216 60 60)"
              />
              <use
                href="#hero-blossom-petal-shape"
                transform="rotate(288 60 60)"
              />
            </g>
            <g class="hero-blossom-outline">
              <use href="#hero-blossom-petal-shape" />
              <use
                href="#hero-blossom-petal-shape"
                transform="rotate(72 60 60)"
              />
              <use
                href="#hero-blossom-petal-shape"
                transform="rotate(144 60 60)"
              />
              <use
                href="#hero-blossom-petal-shape"
                transform="rotate(216 60 60)"
              />
              <use
                href="#hero-blossom-petal-shape"
                transform="rotate(288 60 60)"
              />
            </g>
            <g class="hero-blossom-core">
              <circle cx="60" cy="60" r="2" />
            </g>
            <g class="hero-blossom-stamens">
              <path d="M60 52Q52 41 60 30" />
              <path d="M60 52Q52 41 60 30" transform="rotate(72 60 60)" />
              <path d="M60 52Q52 41 60 30" transform="rotate(144 60 60)" />
              <path d="M60 52Q52 41 60 30" transform="rotate(216 60 60)" />
              <path d="M60 52Q52 41 60 30" transform="rotate(288 60 60)" />
              <circle cx="60" cy="30" r="2.8" />
              <circle cx="60" cy="30" r="2.8" transform="rotate(72 60 60)" />
              <circle cx="60" cy="30" r="2.8" transform="rotate(144 60 60)" />
              <circle cx="60" cy="30" r="2.8" transform="rotate(216 60 60)" />
              <circle cx="60" cy="30" r="2.8" transform="rotate(288 60 60)" />
            </g>
          </svg>
        </div>

        <div v-if="!sysStore.loading" class="hero-content">
          <span class="hero-greeting">{{ t("content.welcome") }}</span>
          <span class="hero-value">{{ t("content.mmrs") }}</span>
          <span
            v-if="sysStore.device.model && sysStore.device.model !== '-'"
            class="hero-subtitle"
          >
            {{ sysStore.device.model }}
          </span>
        </div>

        <div v-else class="skeleton-col">
          <Skeleton class="skeleton-hero-label" />
          <Skeleton class="skeleton-hero-title" />
        </div>
      </div>

      <div class="metrics-row">
        <div class="metric-card">
          <template v-if="moduleStatsReady">
            <span class="metric-value">{{ mountedCount }}</span>
            <span class="metric-label">{{ t("status.moduleActive") }}</span>
          </template>
          <Skeleton v-else class="skeleton-metric" />
          <div class="metric-icon-bg">
            <svg viewBox="0 0 24 24">
              <path :d="ICONS.modules" />
            </svg>
          </div>
        </div>

        <div class="metric-card">
          <template v-if="!sysStore.loading">
            <span class="metric-value">
              {{ configStore.config.mountsource }}
            </span>
            <span class="metric-label">{{ t("status.mountSource") }}</span>
          </template>
          <Skeleton v-else class="skeleton-metric" />
          <div class="metric-icon-bg">
            <svg viewBox="0 0 24 24">
              <path :d="ICONS.ksu" />
            </svg>
          </div>
        </div>
      </div>

      <div class="info-card">
        <div class="card-title">{{ t("status.sysInfoTitle") }}</div>

        <div class="info-row">
          <span class="info-key">{{ t("status.kernelLabel") }}</span>
          <template v-if="!sysStore.loading">
            <span class="info-val">
              {{ sysStore.systemInfo.kernel ?? "-" }}
            </span>
          </template>
          <Skeleton v-else class="skeleton-info-wide" />
        </div>

        <div class="info-row">
          <span class="info-key">{{ t("status.selinuxLabel") }}</span>
          <template v-if="!sysStore.loading">
            <span class="info-val">
              {{ sysStore.systemInfo.selinux ?? "-" }}
            </span>
          </template>
          <Skeleton v-else class="skeleton-info-narrow" />
        </div>
      </div>
    </div>

    <BottomActions>
      <div class="spacer" />
      <button
        class="action-btn-icon"
        @click="sysStore.loadStatus"
        :disabled="sysStore.loading"
        :title="t('status.refresh')"
      >
        <svg viewBox="0 0 24 24" width="24" height="24">
          <path :d="ICONS.refresh" />
        </svg>
      </button>
    </BottomActions>

    <div
      v-if="showRebootConfirm"
      class="dialog-overlay"
      @click.self="showRebootConfirm = false"
    >
      <div class="dialog-content">
        <div class="dialog-headline">{{ t("common.rebootTitle") }}</div>
        <div class="dialog-body">{{ t("common.rebootConfirm") }}</div>
        <div class="dialog-actions">
          <button class="dialog-btn-text" @click="showRebootConfirm = false">
            {{ t("common.cancel") }}
          </button>
          <button class="dialog-btn-text" @click="reboot">
            {{ t("common.reboot") }}
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.dashboard-grid {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding-bottom: 24px;
}

.hero-card {
  --hero-card-bg: var(--md-sys-color-primary);
  --hero-card-fg: var(--md-sys-color-on-primary);
  --hero-subtitle-bg: rgba(255, 255, 255, 0.2);
  --hero-subtitle-border: rgba(255, 255, 255, 0.3);
  --hero-star-opacity: 0.26;
  --hero-blossom-petal: var(--md-sys-color-error-container);
  --hero-blossom-core: var(--md-sys-color-error);
  --hero-blossom-seed: var(--md-sys-color-on-error-container);
  --hero-blossom-outline: var(--md-sys-color-on-error-container);
  background-color: var(--hero-card-bg);
  color: var(--hero-card-fg);
  border-radius: 28px;
  padding: 32px 28px;
  position: relative;
  overflow: hidden;
  display: flex;
  align-items: flex-start;
  justify-content: flex-start;
  min-height: 200px;
  border: none;
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.12);
  user-select: none;
}

.hero-bg-decoration {
  position: absolute;
  inset: 0;
  pointer-events: none;
  z-index: 0;
}

.hero-corner-blossom {
  position: absolute;
  right: -8px;
  bottom: -12px;
  width: 138px;
  height: 138px;
  opacity: var(--hero-star-opacity);
  transform: rotate(-18deg);
  transform-origin: center;
}

.hero-blossom-petals {
  fill: var(--hero-blossom-petal);
}

.hero-blossom-outline {
  fill: none;
  mask: url(#hero-blossom-outline-mask);
  stroke: color-mix(in srgb, var(--hero-blossom-outline) 42%, transparent);
  stroke-width: 1.15;
  stroke-linejoin: round;
}

.hero-blossom-core circle {
  fill: var(--hero-blossom-core);
}

.hero-blossom-stamens path {
  fill: none;
  stroke: var(--hero-blossom-seed);
  stroke-width: 1.35;
  stroke-linecap: round;
}

.hero-blossom-stamens circle {
  fill: var(--hero-blossom-seed);
}

.hero-content {
  display: flex;
  flex-direction: column;
  justify-content: flex-end;
  z-index: 1;
  width: 100%;
  max-width: 100%;
  text-align: left;
  gap: 8px;
}

.hero-greeting {
  font-size: 16px;
  font-weight: 500;
  opacity: 0.9;
  letter-spacing: 0.5px;
}

.hero-value {
  font-size: clamp(34px, 5vw, 44px);
  line-height: 1.1;
  font-weight: 700;
  letter-spacing: -0.02em;
}

.hero-subtitle {
  font-size: 14px;
  font-weight: 500;
  opacity: 1;
  background: var(--hero-subtitle-bg);
  padding: 4px 16px;
  border-radius: 20px;
  width: fit-content;
  backdrop-filter: blur(4px);
  margin-top: 4px;
  color: var(--hero-card-fg);
  border: 1px solid var(--hero-subtitle-border);
}

.metrics-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

.metric-card {
  background-color: var(--md-sys-color-surface-container);
  border-radius: 20px;
  padding: 20px;
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  justify-content: space-between;
  gap: 16px;
  position: relative;
  overflow: hidden;
}

.metric-icon-bg {
  position: absolute;
  bottom: -10px;
  right: -10px;
  opacity: 0.05;
  color: var(--md-sys-color-on-surface);
}

.metric-icon-bg svg {
  width: 80px;
  height: 80px;
  fill: currentColor;
}

.metric-value {
  font-size: 32px;
  font-weight: 600;
  color: var(--md-sys-color-primary);
  line-height: 1;
}

.metric-label {
  font-size: 13px;
  font-weight: 500;
  color: var(--md-sys-color-on-surface-variant);
}

.card-title {
  font-size: 16px;
  font-weight: 500;
  color: var(--md-sys-color-on-surface);
  margin-bottom: 4px;
}

.info-card {
  background-color: var(--md-sys-color-surface-container-low);
  border-radius: 24px;
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.info-row {
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 1px solid var(--md-sys-color-outline-variant);
  padding-bottom: 12px;
  margin-bottom: -8px;
}

.info-row:last-child {
  border-bottom: none;
  padding-bottom: 0;
  margin-bottom: 0;
}

.info-key {
  font-size: 14px;
  color: var(--md-sys-color-on-surface-variant);
}

.info-val {
  font-size: 14px;
  font-weight: 500;
  color: var(--md-sys-color-on-surface);
  font-family: var(--md-ref-typeface-mono);
  text-align: right;
  max-width: 60%;
  word-break: break-all;
}

.skeleton-col {
  display: flex;
  flex-direction: column;
  gap: 8px;
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

.action-btn-icon:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.dialog-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
}

.dialog-content {
  background-color: var(--md-sys-color-surface-container-highest);
  border-radius: 28px;
  padding: 24px;
  width: 90%;
  max-width: 320px;
}

.dialog-headline {
  font-size: 20px;
  font-weight: 500;
  color: var(--md-sys-color-on-surface);
  margin-bottom: 16px;
}

.dialog-body {
  font-size: 14px;
  color: var(--md-sys-color-on-surface-variant);
  margin-bottom: 24px;
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.dialog-btn-text {
  padding: 8px 16px;
  font-size: 14px;
  font-weight: 500;
  color: var(--md-sys-color-primary);
  background: transparent;
  border: none;
  cursor: pointer;
  border-radius: 8px;
  transition: background-color 0.2s;
}

.dialog-btn-text:hover {
  background-color: rgba(103, 80, 164, 0.1);
}

@media (prefers-color-scheme: dark) {
  .hero-card {
    --hero-card-bg: var(--md-sys-color-primary-container);
    --hero-card-fg: var(--md-sys-color-on-primary-container);
    --hero-subtitle-bg: rgba(255, 255, 255, 0.08);
    --hero-subtitle-border: rgba(255, 255, 255, 0.12);
    --hero-star-opacity: 0.2;
    --hero-blossom-petal: var(--md-sys-color-error-container);
    --hero-blossom-core: var(--md-sys-color-error);
    --hero-blossom-seed: var(--md-sys-color-on-error-container);
    --hero-blossom-outline: var(--md-sys-color-on-error-container);
    box-shadow: 0 12px 32px rgba(0, 0, 0, 0.28);
  }
}
</style>
