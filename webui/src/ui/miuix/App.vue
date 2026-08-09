<!--

    Copyright (C) 2026 meta-magic_mount-rs developers
    SPDX-License-Identifier: GPL-v3

-->
<script setup lang="ts">
import { computed, onBeforeUnmount } from "vue";
import { useI18n } from "vue-i18n";
import { showSnackbar } from "miuix-vue";
import { uiStore } from "../../lib/stores/uiStore";
import MiuixLayout from "./MiuixLayout.vue";
import StatusPage from "./pages/status.vue";
import ConfigPage from "./pages/config.vue";
import ModulesPage from "./pages/modules.vue";
import AboutPage from "./pages/about.vue";

const { t } = useI18n();
const pages = [StatusPage, ConfigPage, ModulesPage, AboutPage];
const navindex = computed({
  get: () => uiStore.navindex,
  set: (val: number) => uiStore.setNavindex(val),
});
const activepage = computed(() => pages[navindex.value]);
const titles = computed(() => [
  t("tabs.status"),
  t("tabs.config"),
  t("tabs.modules"),
  t("tabs.info"),
]);

uiStore.setToastHandler((text) => showSnackbar({ message: text }));
onBeforeUnmount(() => uiStore.setToastHandler());
</script>

<template>
  <MiuixLayout
    v-model:navindex="navindex"
    :activepage="activepage"
    :titles="titles"
  />
</template>
