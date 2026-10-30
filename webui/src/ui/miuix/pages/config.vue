<!--
Copyright (C) 2026 meta-magic_mount-rs developers
SPDX-License-Identifier: GPL-v3
-->

<script setup lang="ts">
import { ref, onMounted, computed, watch } from "vue";
import { useI18n } from "vue-i18n";
import {
  getCurrentLangIndex,
  switchLocale,
  getSupportedLocales,
} from "../../../locales";
import {
  MiuixCard,
  MiuixSmallTitle,
  MiuixSwitch,
  MiuixButton,
  MiuixInput,
  MiuixBasicComponent,
  MiuixIcon,
  MiuixIconButton,
  showSnackbar,
} from "miuix-vue";
import {
  Translate,
  Theme,
  FolderFill,
  Layers,
  Add,
  Link,
  Photos,
} from "miuix-vue/icons";

import RemoveableLabel from "../components/RemoveableLabel.vue";
import MiuixDropdownPreference from "../components/DropdownPreference.vue";

import { configStore } from "../../../lib/stores/configStore";
import { DEFAULT_CONFIG } from "../../../lib/constants";
import { uiStore } from "../../../lib/stores/uiStore";
import { isValidExtraMount } from "../../../lib/extraMount";

const { t } = useI18n();

const display_list = ref<string[]>([]);
const lang_code = ref<string[]>([]);
const lang_dropdown_index = ref(0);
const partition = ref("");
const extraMount = ref("");

const current_lang = ref(0);

const styleOptions = ["MiuiX", "Material Design 3"];
const styleslist: ("miuix" | "md3")[] = ["miuix", "md3"];
const styles = computed({
  get: () => styleslist.indexOf(uiStore.uiStyle),
  set: (val: number) => {
    uiStore.setUiStyle(styleslist[val]);
  },
});

getCurrentLangIndex().then((val) => (current_lang.value = val));
const language_set = computed({
  get: () => current_lang.value,
  set: (val: number) => {
    current_lang.value = val;
    switchLocale(lang_code.value[val]);
  },
});

const initialConfigStr = ref("");

const isDirty = computed(() => {
  if (!initialConfigStr.value) return false;
  return JSON.stringify(configStore.config) !== initialConfigStr.value;
});

watch(
  () => configStore.config,
  () => {
    if (
      !configStore.loading &&
      (!initialConfigStr.value ||
        initialConfigStr.value === JSON.stringify(configStore.config))
    ) {
      initialConfigStr.value = JSON.stringify(configStore.config);
    }
  },
  { deep: true },
);

const mountSource = computed({
  get: () => configStore.config.mountsource,
  set: (val) => {
    configStore.setConfig({ ...configStore.config, mountsource: val });
  },
});

const umountEnabled = computed({
  get: () => configStore.config.umount,
  set: (val) => {
    configStore.setConfig({ ...configStore.config, umount: val });
  },
});

const monetTheme = computed({
  get: () => uiStore.monetEnabled,
  set: (val) => {
    uiStore.setMonetEnabled(val);
  },
});

onMounted(async () => {
  const supported_locales = await getSupportedLocales();
  const lang_index = await getCurrentLangIndex();
  lang_dropdown_index.value = lang_index;
  display_list.value = supported_locales.map((l) => l.display);
  lang_code.value = supported_locales.map((l) => l.code);

  await configStore.loadConfig();
});

function handle_add_partition() {
  configStore.config.partitions.push(partition.value);
  partition.value = "";
  updateConfig("partitions", configStore.config.partitions);
}

function addExtraMount() {
  const name = extraMount.value.trim();
  if (!isValidExtraMount(name)) {
    showSnackbar({ message: t("config.extraMountInvalid") });
    return;
  }
  if (!configStore.config.extra_mount.includes(name)) {
    updateConfig("extra_mount", [...configStore.config.extra_mount, name]);
  }
  extraMount.value = "";
}

function removeExtraMount(index: number) {
  updateConfig(
    "extra_mount",
    configStore.config.extra_mount.filter((_, i) => i !== index),
  );
}

function removePartition(index: number) {
  configStore.config.partitions.splice(index, 1);
  updateConfig("partitions", configStore.config.partitions);
}

async function saveConfig() {
  const success = await configStore.saveConfig();
  if (success) {
    initialConfigStr.value = JSON.stringify(configStore.config);
  }
  showSnackbar({
    message: success ? t("config.saveSuccess") : t("config.saveFailed"),
  });
}

function resetConfig() {
  configStore.setConfig({ ...DEFAULT_CONFIG });
  showSnackbar({
    message: t("config.loadDefault"),
  });
}

function updateConfig<K extends keyof typeof configStore.config>(
  key: K,
  value: (typeof configStore.config)[K],
) {
  configStore.setConfig({ ...configStore.config, [key]: value });
}
</script>

<template>
  <div class="page">
    <MiuixSmallTitle text="WebUI" />
    <MiuixCard class="ex-card">
      <MiuixDropdownPreference
        :title="t('common.language')"
        v-model="language_set"
        :items="display_list"
      >
        <template #start>
          <MiuixIcon :icon="Translate" />
        </template>
      </MiuixDropdownPreference>
      <MiuixDropdownPreference
        :title="t('config.uiStyle')"
        :items="styleOptions"
        v-model="styles"
      >
        <template #start>
          <MiuixIcon :icon="Theme" />
        </template>
      </MiuixDropdownPreference>
      <MiuixBasicComponent :title="t('config.monetTheme')">
        <template #start>
          <MiuixIcon :icon="Photos" />
        </template>
        <template #end>
          <MiuixSwitch v-model="monetTheme" />
        </template>
      </MiuixBasicComponent>
    </MiuixCard>

    <MiuixSmallTitle :text="t('tabs.config')" />
    <MiuixCard class="ex-card">
      <MiuixBasicComponent
        :title="t('config.umountLabel')"
        :summary="umountEnabled ? t('config.umountOn') : t('config.umountOff')"
      >
        <template #start>
          <MiuixIcon :icon="Link" />
        </template>
        <template #end>
          <MiuixSwitch v-model="umountEnabled" />
        </template>
      </MiuixBasicComponent>
    </MiuixCard>
    <MiuixCard class="ex-card">
      <MiuixBasicComponent
        :title="t('config.mountSource')"
        :summary="t('config.mountSourceDesc')"
      >
        <template #start>
          <MiuixIcon :icon="FolderFill" />
        </template>
      </MiuixBasicComponent>
      <div style="padding: 0 16px 16px">
        <MiuixInput
          v-model="mountSource"
          :label="t('config.mountSource')"
          single-line
        />
      </div>
    </MiuixCard>

    <MiuixCard class="ex-card">
      <MiuixBasicComponent
        :title="t('config.partitions')"
        :summary="t('config.partitionsDesc')"
      >
        <template #start>
          <MiuixIcon :icon="Layers" />
        </template>
      </MiuixBasicComponent>
      <div v-if="configStore.config.partitions.length > 0" class="chip-list">
        <RemoveableLabel
          v-for="(partition, index) in configStore.config.partitions"
          :key="index"
          :text="partition"
          @remove="removePartition(index)"
        />
      </div>
      <div style="display: flex; padding: 0 16px 16px">
        <MiuixInput
          v-model="partition"
          label="e.g. product,system_ext..."
          single-line
        />
        <MiuixIconButton v-if="partition" @click="handle_add_partition()">
          <MiuixIcon :icon="Add" :size="24" />
        </MiuixIconButton>
      </div>
    </MiuixCard>
    <MiuixCard class="ex-card">
      <MiuixBasicComponent
        :title="t('config.extraMount')"
        :summary="t('config.extraMountDesc')"
      >
        <template #start>
          <MiuixIcon :icon="Layers" />
        </template>
      </MiuixBasicComponent>
      <div v-if="configStore.config.extra_mount.length > 0" class="chip-list">
        <RemoveableLabel
          v-for="(name, index) in configStore.config.extra_mount"
          :key="index"
          :text="name"
          @remove="removeExtraMount(index)"
        />
      </div>
      <div style="display: flex; padding: 0 16px 16px">
        <MiuixInput
          v-model="extraMount"
          :label="t('config.extraMount')"
          placeholder="my_product"
          single-line
          @keydown.enter.prevent="addExtraMount"
        />
        <MiuixIconButton
          v-if="extraMount"
          :aria-label="t('config.extraMount')"
          @click="addExtraMount"
        >
          <MiuixIcon :icon="Add" :size="24" />
        </MiuixIconButton>
      </div>
    </MiuixCard>
    <MiuixCard class="ex-card">
      <div style="padding: 12px; display: flex; gap: 12px">
        <MiuixButton style="flex: 1" @click="resetConfig">
          {{ t("config.reset") }}
        </MiuixButton>
        <MiuixButton
          type="primary"
          style="flex: 1"
          :disabled="configStore.saving || !isDirty"
          @click="saveConfig"
        >
          {{ t("config.save") }}
        </MiuixButton>
      </div>
    </MiuixCard>
  </div>
</template>

<style scoped>
.ex-card {
  margin: 0 12px 12px;
}
.ex-basic-row {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}
.ex-grow {
  flex: 1;
}
.chip-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  padding: 16px;
}
</style>
