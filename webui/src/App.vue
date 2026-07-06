<!--

    Copyright (C) 2026 meta-magic_mount-rs developers
    SPDX-License-Identifier: GPL-v3

-->
<script setup lang="ts">
import { ref, computed, watch, onMounted, onBeforeUnmount } from "vue";
import { useI18n } from "vue-i18n";
import {
  MiuixSnackbarHost,
  MiuixScrollArea,
  MiuixIcon,
  MiuixNavigationBar,
  MiuixTopAppBar,
  MiuixIconButton,
  MiuixButton,
  MiuixDialog,
} from "miuix-vue";
import {
  ScreenMirroring,
  Settings,
  Info,
  Folder,
  Close2,
} from "miuix-vue/icons";

import status from "./page/status.vue";
import config from "./page/config.vue";
import modules from "./page/modules.vue";
import about from "./page/about.vue";
import { sysStore } from "./lib/stores/sysStore";

const { t } = useI18n();

const rebootreq_click = ref(false);

const pages = [status, config, modules, about];
const titles = computed(() => [
  t("tabs.status"),
  t("tabs.config"),
  t("tabs.modules"),
  t("tabs.info"),
]);
const navItems = computed(() => titles.value.map((label) => ({ label })));
const navicoms = [ScreenMirroring, Settings, Folder, Info];

const navindex = ref(0);
const activepage = computed(() => pages[navindex.value]);

interface Scroller {
  getScrollTop: () => number;
  setScrollTop: (top: number) => void;
}
const scrollerRef = ref<Scroller | null>(null);
const scrollPositions = new Map<number, number>();

watch(
  navindex,
  (_next, prev) => {
    scrollPositions.set(prev, scrollerRef.value?.getScrollTop() ?? 0);
  },
  { flush: "pre" },
);

function onPageEnter(): void {
  scrollerRef.value?.setScrollTop(scrollPositions.get(navindex.value) ?? 0);
}

function reboot_system(): void {
  sysStore.rebootDevice();
  rebootreq_click.value = false;
}

const bottomBarRef = ref<HTMLElement | null>(null);
let barObserver: ResizeObserver | null = null;

function syncSnackbarInset(): void {
  const h = bottomBarRef.value?.offsetHeight ?? 0;
  document.documentElement.style.setProperty(
    "--m-snackbar-inset-bottom",
    `${h}px`,
  );
}

onMounted(() => {
  if (bottomBarRef.value) {
    barObserver = new ResizeObserver(syncSnackbarInset);
    barObserver.observe(bottomBarRef.value);
  }
  syncSnackbarInset();
});

onBeforeUnmount(() => {
  barObserver?.disconnect();
  document.documentElement.style.removeProperty("--m-snackbar-inset-bottom");
});
</script>

<template>
  <div class="app">
    <MiuixScrollArea ref="scrollerRef" class="app__body">
      <MiuixTopAppBar
        :large="false"
        :title="t('common.appName')"
        class="app__top-app-bar"
      >
        <template #actions>
          <MiuixIconButton aria-label="Reboot" @click="rebootreq_click = true">
            <MiuixIcon :icon="Close2" :size="24" />
          </MiuixIconButton>
        </template>
      </MiuixTopAppBar>

      <Transition name="page" mode="out-in" @enter="onPageEnter">
        <KeepAlive>
          <component :is="activepage" :key="navindex" v-if="activepage" />
        </KeepAlive>
      </Transition>
    </MiuixScrollArea>

    <div ref="bottomBarRef" class="app__bottom">
      <MiuixNavigationBar v-model="navindex" :items="navItems">
        <template #icon="{ index }">
          <MiuixIcon :icon="navicoms[index]" :size="26" />
        </template>
      </MiuixNavigationBar>
    </div>
  </div>

  <MiuixSnackbarHost />

  <MiuixDialog
    v-model="rebootreq_click"
    :title="t('common.rebootTitle')"
    :summary="t('common.rebootConfirm')"
    @close="rebootreq_click = false"
  >
    <template #default="{ close }">
      <div class="ex-dialog-actions">
        <MiuixButton class="ex-grow" @click="close">
          {{ t("common.cancel") }}
        </MiuixButton>
        <MiuixButton class="ex-grow" type="primary" @click="reboot_system">
          {{ t("common.reboot") }}
        </MiuixButton>
      </div>
    </template>
  </MiuixDialog>
</template>

<style scoped>
.app {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: var(--m-color-surface);
}
.app__body {
  flex: 1;
  min-height: 0;
  overflow: hidden;
  --m-scroll-area-inset-top: 52px;
}
.app__top-app-bar {
  padding-top: var(--top-inset);
}
.app__bottom {
  flex: none;
  z-index: 10;
  padding-bottom: var(--bottom-inset);
}

.page-enter-active,
.page-leave-active {
  transition: opacity 0.18s ease;
}
.page-enter-from,
.page-leave-to {
  opacity: 0;
}

.ex-dialog-actions {
  display: flex;
  gap: 12px;
}

.ex-grow {
  flex: 1;
}
</style>
