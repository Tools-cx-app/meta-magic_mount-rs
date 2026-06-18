<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { useI18n } from "vue-i18n";
import {
  MiuixSearchBar,
  MiuixCard,
  MiuixText,
  MiuixBasicComponent,
} from "miuix-vue";
import { Motion, AnimatePresence } from "motion-v";
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
      <div v-for="module in filterModules" :key="module.id">
        <MiuixCard class="ex-card">
          <MiuixBasicComponent
            :title="module.name"
            :summary="module.id + ' ' + module.version"
            :clickable="true"
            @click="module.bottomopen = !module.bottomopen"
          >
            <template #end>
              <MiuixText
                type="body2"
                color="var(--m-color-on-surface-variant-actions)"
              >
                {{ module.is_mounted ? "MOUNTED" : "UNMOUNTED" }}
              </MiuixText>
            </template>
          </MiuixBasicComponent>
          <AnimatePresence :initial="false">
            <Motion
              v-if="module.bottomopen"
              class="ex-expand"
              :initial="{ height: 0, opacity: 0 }"
              :animate="{ height: 'auto', opacity: 1 }"
              :exit="{ height: 0, opacity: 0 }"
              :transition="expandSpring"
            >
              <MiuixBasicComponent
                :title="t('modules.descriptionLabel')"
                :summary="module.description"
              />
              <MiuixBasicComponent
                :title="t('modules.authorLabel')"
                :summary="module.author"
              />
            </Motion>
          </AnimatePresence>
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
