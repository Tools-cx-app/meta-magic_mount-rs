<!--

    Copyright (C) 2026 meta-magic_mount-rs developers
    SPDX-License-Identifier: GPL-v3

-->
<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { useI18n } from "vue-i18n";
import {
  MiuixSearchBar,
  MiuixCard,
  MiuixText,
  MiuixIcon,
  MiuixBasicComponent,
  MiuixProgressIndicator,
} from "miuix-vue";
import { Help } from "miuix-vue/icons";
import Label from "../components/Label.vue";
import { moduleStore } from "../../../lib/stores/moduleStore";

const { t } = useI18n();

const searchQuery = ref("");
const searchexpanded = ref(false);

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
    <div v-if="moduleStore.loading" class="loading-wrapper">
      <div class="loading-content">
        <MiuixProgressIndicator
          type="infinite"
          color="var(--m-color-on-background-variant)"
          style="flex: 1"
        />
        <MiuixText
          color="var(--m-color-on-surface-variant-actions)"
          style="flex: 1"
        >
          {{ t("modules.scanning") }}
        </MiuixText>
      </div>
    </div>

    <div
      v-else-if="moduleStore.modules.length === 0 || filterModules.length === 0"
      class="notfound-wrapper"
    >
      <div class="loading-content">
        <MiuixIcon :icon="Help" size="90" />
        <MiuixText color="var(--m-color-on-surface-variant-actions)">
          {{ t("modules.emptyState") }}
        </MiuixText>
      </div>
    </div>

    <div v-else>
      <div
        v-for="module in filterModules
          .slice()
          .sort(
            (a, b) =>
              (b.is_mounted === true ? 1 : 0) - (a.is_mounted === true ? 1 : 0),
          )"
        :key="module.id"
      >
        <MiuixCard show-indication class="ex-card">
          <MiuixBasicComponent
            :title="module.name"
            :summary="module.author + ' ' + module.version"
          >
            <template #end>
              <Label
                v-if="module.is_mounted"
                bgColor="var(--m-color-tertiary-container)"
                textColor="var(--m-color-on-tertiary-container)"
              >
                MOUNTED
              </Label>
              <Label
                v-else
                bgColor="var(--m-color-umount-label-bg)"
                textColor="var(--m-color-umount-label-text)"
              >
                UNMOUNTED
              </Label>
            </template>
          </MiuixBasicComponent>
          <MiuixText
            type="body2"
            color="var(--m-color-on-surface-variant-actions)"
            style="margin: 0 16px 12px"
          >
            {{
              module.description
                ? module.description
                : t("modules.noDescriptionLabel")
            }}
          </MiuixText>
        </MiuixCard>
      </div>
    </div>
  </div>
</template>

<style>
:root,
.m-theme-light {
  --m-color-umount-label-bg: rgba(0, 0, 0, 0.3);
  --m-color-umount-label-text: rgba(255, 255, 255, 0.8);
}
.m-theme-dark {
  --m-color-umount-label-bg: rgba(255, 255, 255, 0.3);
  --m-color-umount-label-text: rgba(0, 0, 0, 0.4);
}

.icon-search {
  padding: 0 0 12px;
}
.ex-card {
  margin: 0 12px 12px;
}

.loading-wrapper {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  display: flex;
  text-align: center;
  align-items: center;
  justify-content: center;
}

.loading-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 12px;
}

.notfound-wrapper {
  position: absolute;
  left: 38%;
  top: 45%;
  display: flex;
  text-align: center;
  align-items: center;
  justify-content: center;
}
</style>
