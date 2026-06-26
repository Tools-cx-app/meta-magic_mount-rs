<!--

    Copyright (C) 2026 meta-magic_mount-rs developers
    SPDX-License-Identifier: Apache-2.0

-->
<script setup lang="ts">
import { ref, onMounted, computed, watch } from "vue";
import { useI18n } from "vue-i18n";
import {
  getCurrentLangIndex,
  switchLocale,
  getSupportedLocales,
} from "../locales";
import {
  MiuixCard,
  MiuixDialog,
  MiuixSmallTitle,
  MiuixSwitch,
  MiuixButton,
  MiuixDropdownPreference,
  MiuixInput,
  MiuixBasicComponent,
  MiuixIcon,
  MiuixIconButton,
  showSnackbar,
} from "miuix-vue";
import {
  FolderFill,
  Delete,
  Layers,
  MoveFile,
  Add,
  Link,
} from "miuix-vue/icons";

import RemoveableLabel from "../components/RemoveableLabel.vue";
import BindCard from "../components/BindCard.vue";
import IgnoredCard from "../components/IgnoredCard.vue";

import { configStore } from "../lib/stores/configStore";
import { DEFAULT_CONFIG } from "../lib/constants";
import type { CustomMount } from "../lib/types";

const { t } = useI18n();

const display_list = ref<string[]>([]);
const lang_code = ref<string[]>([]);
const lang_dropdown_index = ref(0);
const partition = ref("");
const ignorepath = ref("");

const current_lang = ref(0);

getCurrentLangIndex().then((index) => (current_lang.value = index));

const customMountDraft = ref<CustomMount>({ source: "", target: "" });
const editingCustomMountIndex = ref<number | null>(null);
const dialogVisible = ref(false);

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

onMounted(async () => {
  const supported_locales = await getSupportedLocales();
  const lang_index = await getCurrentLangIndex();
  lang_dropdown_index.value = lang_index;
  display_list.value = supported_locales.map((l) => l.display);
  lang_code.value = supported_locales.map((l) => l.code);

  await configStore.loadConfig();
});

async function handleChange(value: number) {
  await switchLocale(lang_code.value[value]);
  window.location.reload();
}

function handle_add_partition() {
  configStore.config.partitions.push(partition.value);
  partition.value = "";
  updateConfig("partitions", configStore.config.partitions);
}

function handle_add_ignorepath() {
  configStore.config.ignoreList.push(ignorepath.value);
  ignorepath.value = "";
  updateConfig("ignoreList", configStore.config.ignoreList);
}

function removePartition(index: number) {
  configStore.config.partitions.splice(index, 1);
  updateConfig("partitions", configStore.config.partitions);
}

function removeIgnorepath(index: number) {
  configStore.config.ignoreList.splice(index, 1);
  updateConfig("ignoreList", configStore.config.ignoreList);
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

function openAddCustomMountDialog() {
  editingCustomMountIndex.value = null;
  customMountDraft.value = { source: "", target: "" };
  dialogVisible.value = true;
}

function openEditCustomMountDialog(index: number) {
  editingCustomMountIndex.value = index;
  customMountDraft.value = { ...configStore.config.customMounts[index] };
  dialogVisible.value = true;
}
function closeCustomMountDialog() {
  dialogVisible.value = false;
}

function deleteCustomMountDialog() {
  if (editingCustomMountIndex.value !== null) {
    updateConfig(
      "customMounts",
      configStore.config.customMounts.filter(
        (_, index) => index !== editingCustomMountIndex.value,
      ),
    );
    closeCustomMountDialog();
  }
}

function saveCustomMountDialog() {
  const draft = {
    source: customMountDraft.value.source.trim(),
    target: customMountDraft.value.target.trim(),
  };

  if (!draft.source || !draft.target) {
    return;
  }

  if (editingCustomMountIndex.value === null) {
    updateConfig("customMounts", [...configStore.config.customMounts, draft]);
  } else {
    updateConfig(
      "customMounts",
      configStore.config.customMounts.map((mount, index) =>
        index === editingCustomMountIndex.value ? draft : mount,
      ),
    );
  }

  closeCustomMountDialog();
}
</script>

<template>
  <div class="page">
    <MiuixSmallTitle text="WebUI" />
    <MiuixCard class="ex-card">
      <MiuixDropdownPreference
        :title="t('common.language')"
        :summary="lang_code[lang_dropdown_index]"
        v-model="lang_dropdown_index"
        :items="display_list"
      />
      <div style="padding: 12px">
        <MiuixButton
          type="primary"
          :disabled="lang_dropdown_index === current_lang"
          @click="handleChange(lang_dropdown_index)"
        >
          {{ t("config.apply") }}
        </MiuixButton>
      </div>
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
        :title="t('config.ignoreList')"
        :summary="t('config.ignoreListDesc')"
      >
        <template #start>
          <MiuixIcon :icon="Delete" />
        </template>
      </MiuixBasicComponent>
      <div v-if="configStore.config.ignoreList.length > 0" class="chip-list">
        <IgnoredCard
          v-for="(path, index) in configStore.config.ignoreList"
          :key="index"
          :path="path"
          @remove="removeIgnorepath(index)"
        />
      </div>
      <div style="display: flex; padding: 0 16px 16px">
        <MiuixInput
          v-model="ignorepath"
          label="/data/adb/modules/..."
          single-line
        />
        <MiuixIconButton v-if="ignorepath" @click="handle_add_ignorepath()">
          <MiuixIcon :icon="Add" :size="24" />
        </MiuixIconButton>
      </div>
    </MiuixCard>
    <MiuixCard class="ex-card">
      <MiuixBasicComponent
        :title="t('config.customMounts')"
        :summary="t('config.customMountsDesc')"
      >
        <template #start>
          <MiuixIcon :icon="MoveFile" />
        </template>
        <template #end>
          <MiuixIconButton @click="openAddCustomMountDialog">
            <MiuixIcon :icon="Add" />
          </MiuixIconButton>
        </template>
      </MiuixBasicComponent>
      <div
        v-if="configStore.config.customMounts.length > 0"
        style="padding: 16px"
      >
        <BindCard
          v-for="(mount, index) in configStore.config.customMounts"
          :key="index"
          :source-label="t('config.customMountSource')"
          :source="mount.source"
          :target-label="t('config.customMountTarget')"
          :target="mount.target"
          @edit="openEditCustomMountDialog(index)"
        />
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

    <MiuixDialog
      v-model="dialogVisible"
      :title="
        editingCustomMountIndex === null
          ? t('config.customMountDialogAdd')
          : t('config.customMountDialogEdit')
      "
    >
      <div class="custom-mount-dialog-fields">
        <MiuixInput
          v-model="customMountDraft.source"
          :label="t('config.customMountSource')"
          placeholder="/data/adb/modules/test/bin/unit"
          single-line
          class="full-width-field"
        />
        <MiuixInput
          v-model="customMountDraft.target"
          :label="t('config.customMountTarget')"
          placeholder="/product/bin/unit"
          single-line
          class="full-width-field"
        />
      </div>
      <div class="dialog-actions">
        <div
          v-if="editingCustomMountIndex !== null"
          class="dialog-actions-left"
        >
          <MiuixIconButton @click="deleteCustomMountDialog">
            <MiuixIcon :icon="Delete" :size="24" color="var(--m-color-error)" />
          </MiuixIconButton>
        </div>
        <div class="dialog-actions-right">
          <MiuixButton style="flex: 1" @click="closeCustomMountDialog">
            {{ t("common.cancel") }}
          </MiuixButton>
          <MiuixButton
            style="flex: 1"
            type="primary"
            @click="saveCustomMountDialog"
          >
            {{ t("config.customMountDialogSave") }}
          </MiuixButton>
        </div>
      </div>
    </MiuixDialog>
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
.custom-mount-dialog-fields {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 8px 0;
}

.dialog-actions {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding-top: 16px;
  margin-top: 8px;
  border-top: 1px solid var(--m-color-outline-variant);
}

.dialog-actions-left {
  display: flex;
  gap: 8px;
}

.dialog-actions-right {
  display: flex;
  gap: 8px;
  width: 100%;
}
</style>
