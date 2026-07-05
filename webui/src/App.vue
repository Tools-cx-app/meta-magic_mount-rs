<!--

    Copyright (C) 2026 meta-magic_mount-rs developers
    SPDX-License-Identifier: GPL-v3

-->
<script setup lang="ts">
import { ref, computed } from "vue";
import { useI18n } from "vue-i18n";
import { uiStore } from "./lib/stores/uiStore";
import MiuixLayout from "./layouts/MiuixLayout.vue";
import Md3Layout from "./layouts/Md3Layout.vue";

// MIUIX Page
import status from "./page/status.vue";
import config from "./page/config.vue";
import modules from "./page/modules.vue";
import about from "./page/about.vue";

// md3page
import md3status from "./md3page/status.vue";
import md3config from "./md3page/config.vue";
import md3modules from "./md3page/modules.vue";
import md3about from "./md3page/about.vue";

const { t } = useI18n();

const md3_pages = [md3status, md3config, md3modules, md3about];
const miuix_pages = [status, config, modules, about];
const pages = computed(() =>
  uiStore.uiStyle === "md3" ? md3_pages : miuix_pages,
);
const titles = computed(() => [
  t("tabs.status"),
  t("tabs.config"),
  t("tabs.modules"),
  t("tabs.info"),
]);

const navindex = ref(0);
const activepage = computed(() => pages.value[navindex.value]);

const CurrentLayout = computed(() => {
  return uiStore.uiStyle === "md3" ? Md3Layout : MiuixLayout;
});

const appStyleClass = computed(() => `app--style-${uiStore.uiStyle}`);
</script>

<template>
  <div :class="appStyleClass">
    <component
      :is="CurrentLayout"
      v-model:navindex="navindex"
      :activepage="activepage"
      :titles="titles"
    />
  </div>
</template>

<style scoped>
.app--style-md3 {
  --m-color-surface: var(--m-color-surface);
}
</style>
