<!--

    Copyright (C) 2026 meta-magic_mount-rs developers
    SPDX-License-Identifier: GPL-v3

-->
<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount, type Component } from "vue";
import { useI18n } from "vue-i18n";
import { API } from "../lib/api";
import { ICONS } from "../lib/constants";
import "../md3-theme.css";

const { t } = useI18n();

const isDark = ref(false);
let mediaQuery: MediaQueryList | null = null;

function applyTheme() {
  document.documentElement.classList.toggle("dark", isDark.value);
}

function handleThemeChange(e: MediaQueryListEvent) {
  isDark.value = e.matches;
}

onMounted(() => {
  document.documentElement.classList.add("md3-active");
  mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
  isDark.value = mediaQuery.matches;
  applyTheme();
  mediaQuery.addEventListener("change", handleThemeChange);
});

onBeforeUnmount(() => {
  document.documentElement.classList.remove("md3-active");
  mediaQuery?.removeEventListener("change", handleThemeChange);
});

watch(isDark, () => {
  applyTheme();
});

const props = defineProps<{
  navindex: number;
  activepage: Component;
  titles: string[];
}>();

const emit = defineEmits<{
  (e: "update:navindex", value: number): void;
}>();

const localNavindex = ref(props.navindex);
const showRebootConfirm = ref(false);

watch(
  () => props.navindex,
  (val) => {
    localNavindex.value = val;
  },
);

function handleNavChange(index: number) {
  localNavindex.value = index;
  emit("update:navindex", index);
}

function reboot() {
  showRebootConfirm.value = false;
  void API.reboot();
}

const navIcons = ["home", "settings", "modules", "info"];
</script>

<template>
  <div class="md3-layout">
    <header class="top-bar">
      <div class="top-bar-content">
        <h1 class="screen-title">{{ t("common.appName") }}</h1>
        <div class="top-actions">
          <button
            class="top-action-btn"
            @click="showRebootConfirm = true"
            :title="t('common.reboot')"
          >
            <svg viewBox="0 0 24 24" width="24" height="24">
              <path :d="ICONS.power" />
            </svg>
          </button>
        </div>
      </div>
    </header>

    <main class="md3-layout__main">
      <Transition name="page" mode="out-in">
        <component :is="activepage" :key="navindex" v-if="activepage" />
      </Transition>
    </main>

    <nav class="bottom-nav">
      <button
        v-for="(title, index) in titles"
        :key="index"
        class="nav-tab"
        :class="{ active: localNavindex === index }"
        @click="handleNavChange(index)"
      >
        <div class="icon-container">
          <svg viewBox="0 0 24 24" width="24" height="24">
            <path
              :d="
                localNavindex === index
                  ? ICONS[`${navIcons[index]}_filled` as keyof typeof ICONS] ||
                    ICONS[navIcons[index] as keyof typeof ICONS]
                  : ICONS[navIcons[index] as keyof typeof ICONS]
              "
            />
          </svg>
        </div>
        <span class="label">{{ title }}</span>
      </button>
    </nav>

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
:root {
  --top-inset: var(--window-inset-top, 0px);
  --bottom-inset: var(--window-inset-bottom, 0px);
  background-color: var(--md-sys-color-background);
  color: var(--md-sys-color-on-background);
  font-family: var(--md-ref-typeface-plain);
  font-size: 16px;
  line-height: 1.5;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  overscroll-behavior: none;
}

.md3-layout {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background-color: var(--md-sys-color-background);
}

.top-bar {
  position: sticky;
  top: 0;
  z-index: 100;
  background-color: var(--md-sys-color-surface);
  border-bottom: 1px solid var(--md-sys-color-outline-variant);
  padding-top: var(--top-inset);
}

.top-bar-content {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 16px;
}

.screen-title {
  font-size: 20px;
  font-weight: 500;
  color: var(--md-sys-color-on-surface);
  margin: 0;
}

.top-actions {
  display: flex;
  gap: 8px;
}

.top-action-btn {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  border: none;
  background-color: transparent;
  color: var(--md-sys-color-on-surface-variant);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: background-color 0.2s;
}

.top-action-btn svg {
  fill: currentColor;
}

.top-action-btn:hover {
  background-color: var(--md-sys-color-surface-container-highest);
}

.md3-layout__main {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
  padding-bottom: calc(16px + 64px + var(--bottom-inset, 0px));
  max-width: 800px;
  margin: 0 auto;
  width: 100%;
  box-sizing: border-box;
}

.bottom-nav {
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  display: flex;
  justify-content: space-around;
  align-items: center;
  padding: 8px 0 calc(8px + var(--bottom-inset));
  background-color: var(--md-sys-color-surface);
  color: var(--md-sys-color-on-surface);
  border-top: 1px solid var(--md-sys-color-outline-variant);
  z-index: 90;
  overflow-x: auto;
  scrollbar-width: none;
}

.bottom-nav::-webkit-scrollbar {
  display: none;
}

.nav-tab {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 8px 16px;
  border: none;
  background-color: var(--md-sys-color-surface);
  cursor: pointer;
  transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);
  border-radius: 12px;
  color: var(--md-sys-color-on-surface-variant);
}

.nav-tab.active {
  color: var(--md-sys-color-primary);
}

.icon-container {
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: var(--md-sys-color-surface);
  transition:
    transform 0.2s cubic-bezier(0.4, 0, 0.2, 1),
    background-color 0.2s;
  width: 40px;
  height: 24px;
  border-radius: 16px;
}

.nav-tab.active .icon-container {
  transform: scale(1.1);
  background-color: var(--md-sys-color-primary-container);
}

.icon-container svg {
  fill: currentColor;
}

.label {
  font-size: 11px;
  font-weight: 500;
  transition: font-weight 0.2s;
}

.nav-tab.active .label {
  font-weight: 600;
}

.page-enter-active,
.page-leave-active {
  transition: opacity 0.18s ease;
}

.page-enter-from,
.page-leave-to {
  opacity: 0;
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
</style>
