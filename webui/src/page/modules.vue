<!--

    Copyright (C) 2026 Tools-cx-app <localhost.hutao@gmail.com>
    SPDX-License-Identifier: Apache-2.0

-->
<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { useI18n } from "vue-i18n";
import {
  MiuixSearchBar,
  MiuixCard,
  MiuixText,
  MiuixBasicComponent,
} from "miuix-vue";
import Label from "../components/Label.vue";
import { moduleStore } from "../lib/stores/moduleStore";

const { t } = useI18n();

const searchQuery = ref("");
const searchexpanded = ref(false);

const expandSpring = { type: "spring" as const, stiffness: 400, damping: 40 };

const filterModules = computed(() => {
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
</script>

<template>
  <div class="page">
    <div class="icon-search">
      <MiuixSearchBar
        v-model="searchQuery"
        v-model:expanded="searchexpanded"
        :label="t('modules.searchPlaceholder')"
      ></MiuixSearchBar>
    </div>

    <div v-if="moduleStore.loading" align="center">
      <MiuixText>{{ t("modules.scanning") }}</MiuixText>
    </div>

    <div
      v-else-if="moduleStore.modules.length === 0 || filterModules.length === 0"
      align="center"
    >
      <MiuixText class="ex-card">{{ t("modules.emptyState") }}</MiuixText>
    </div>

    <div v-else>
      <div v-for="module in filterModules.slice().sort((a, b) => (b.is_mounted === true ? 1 : 0) - (a.is_mounted === true ? 1 : 0))" :key="module.id">
        <MiuixCard class="ex-card">
          <MiuixBasicComponent
            :title="module.name"
            :summary="module.author + ' ' + module.version"
          >
            <template #end>
              <Label v-if="module.is_mounted" text="MOUNTED" bgColor="var(--m-color-tertiary-container)" textColor="var(--m-color-on-tertiary-container)" />  
              <Label v-else text="UNMOUNTED" bgColor="var(--m-color-secondary-container)" textColor="var(--m-color-on-secondary-container)" />
            </template>
          </MiuixBasicComponent>
          <MiuixText
            type="body2"
            color="var(--m-color-on-surface-variant-actions)"
            style="margin: 0 16px 12px"
          >
            {{ module.description }}
          </MiuixText>
        </MiuixCard>
      </div>
    </div>
  </div>
</template>

<style scoped>
.icon-search {
  padding: 0 0 6px;
}
.ex-card {
  margin: 0 12px 12px;
}
</style>
