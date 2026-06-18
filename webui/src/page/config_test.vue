<script setup lang="ts">
import { ref, onMounted, computed, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { getCurrentLangIndex, switchLocale, getSupportedLocales } from '../locales';
import { MiuixCard, MiuixButton, MiuixDropdownPreference, MiuixInput, MiuixIcon, MiuixDialog, showSnackbar } from 'miuix-vue';
import { FolderFill, Delete, Layers, MoveFile, Add, Refresh, Ok, Link } from 'miuix-vue/icons';

import CustomIconButton from '../components/CustomIconButton.vue';
import RemoveableLabel from '../components/RemoveableLabel.vue';
import BindCard from '../components/BindCard.vue';

import { configStore } from '../lib/stores/configStore';
import { DEFAULT_CONFIG } from '../lib/constants';
import type { CustomMount } from '../lib/types';

const { t } = useI18n();

const display_list = ref<string[]>([]);
const lang_code = ref<string[]>([]);
const lang_dropdown_index = ref(0);
const partition = ref('');
const ignorepath = ref('');

const initialConfigStr = ref('');
const customMountDraft = ref<CustomMount>({ source: '', target: '' });
const editingCustomMountIndex = ref<number | null>(null);
const dialogVisible = ref(false);

const mountSource = computed({
  get: () => configStore.config.mountsource,
  set: (val) => {
    configStore.setConfig({ ...configStore.config, mountsource: val });
  },
});

const isDirty = computed(() => {
  if (!initialConfigStr.value) return false;
  return JSON.stringify(configStore.config) !== initialConfigStr.value;
});

watch(
  () => configStore.config,
  () => {
    if (!configStore.loading && (!initialConfigStr.value || initialConfigStr.value === JSON.stringify(configStore.config))) {
      initialConfigStr.value = JSON.stringify(configStore.config);
    }
  },
  { deep: true }
);

function updateConfig<K extends keyof typeof configStore.config>(key: K, value: typeof configStore.config[K]) {
  configStore.setConfig({ ...configStore.config, [key]: value });
}

async function save() {
  await configStore.saveConfig();
  initialConfigStr.value = JSON.stringify(configStore.config);
  showSnackbar({ message: t('config.saveSuccess') });
}

async function reload() {
  await configStore.loadConfig();
  initialConfigStr.value = JSON.stringify(configStore.config);
}

function toggleBool(key: keyof typeof configStore.config) {
  const currentValue = configStore.config[key];
  if (typeof currentValue === 'boolean') {
    updateConfig(key, !currentValue as typeof configStore.config[typeof key]);
  }
}

function openAddCustomMountDialog() {
  editingCustomMountIndex.value = null;
  customMountDraft.value = { source: '', target: '' };
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

function saveCustomMountDialog() {
  const draft = {
    source: customMountDraft.value.source.trim(),
    target: customMountDraft.value.target.trim(),
  };

  if (!draft.source || !draft.target) {
    showSnackbar({ message: t('config.customMountEmpty') });
    return;
  }

  if (editingCustomMountIndex.value === null) {
    updateConfig('customMounts', [...configStore.config.customMounts, draft]);
  } else {
    updateConfig(
      'customMounts',
      configStore.config.customMounts.map((mount, index) =>
        index === editingCustomMountIndex.value ? draft : mount
      )
    );
  }

  closeCustomMountDialog();
}

function deleteCustomMountDialog() {
  const index = editingCustomMountIndex.value;
  if (index === null) return;

  updateConfig(
    'customMounts',
    configStore.config.customMounts.filter((_, mountIndex) => mountIndex !== index)
  );
  closeCustomMountDialog();
}

onMounted(async () => {
  const supported_locales = await getSupportedLocales();
  const lang_index = await getCurrentLangIndex();
  lang_dropdown_index.value = lang_index;
  display_list.value = supported_locales.map(l => l.display);
  lang_code.value = supported_locales.map(l => l.code);

  await configStore.loadConfig();
});

function handleChange(value: number) {
  switchLocale(lang_code.value[value]);
  window.location.reload();
}

function handle_add_partition() {
  if (!partition.value.trim()) return;
  updateConfig('partitions', [...configStore.config.partitions, partition.value.trim()]);
  partition.value = '';
}

function handle_add_ignorepath() {
  if (!ignorepath.value.trim()) return;
  updateConfig('ignoreList', [...configStore.config.ignoreList, ignorepath.value.trim()]);
  ignorepath.value = '';
}

function removePartition(index: number) {
  updateConfig('partitions', configStore.config.partitions.filter((_, i) => i !== index));
}

function removeIgnorepath(index: number) {
  updateConfig('ignoreList', configStore.config.ignoreList.filter((_, i) => i !== index));
}

function saveConfig() {
  save();
}

function resetConfig() {
  configStore.setConfig({ ...DEFAULT_CONFIG });
  configStore.loadConfig();
  showSnackbar({ message: t('config.loadDefault') });
}
</script>

<template>
  <div class="page">
    <MiuixDialog
      v-model="dialogVisible"
      :title="editingCustomMountIndex === null ? t('config.customMountDialogAdd') : t('config.customMountDialogEdit')"
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
        <div v-if="editingCustomMountIndex !== null" class="dialog-actions-left">
          <CustomIconButton
            :icon="Delete"
            :size="24"
            color="var(--m-color-error)"
            @click="deleteCustomMountDialog"
          />
        </div>
        <div class="dialog-actions-right">
          <MiuixButton @click="closeCustomMountDialog">{{ t('common.cancel') }}</MiuixButton>
          <MiuixButton type="primary" @click="saveCustomMountDialog">{{ t('config.customMountDialogSave') }}</MiuixButton>
        </div>
      </div>
    </MiuixDialog>

    <div class="config-container">
      <section class="config-group">
        <MiuixCard class="ex-card">
          <MiuixDropdownPreference
            :title="t('common.language')"
            :summary="lang_code[lang_dropdown_index]"
            v-model="lang_dropdown_index"
            :items="display_list"
          />
          <div class="lang-action">
            <MiuixButton type="primary" @click="handleChange(lang_dropdown_index)">
              {{ t('status.refresh') }}
            </MiuixButton>
          </div>
        </MiuixCard>
      </section>

      <section class="config-group">
        <MiuixCard class="ex-card">
          <div class="card-header">
            <div class="card-icon">
              <MiuixIcon :icon="FolderFill" />
            </div>
            <div class="card-text">
              <span class="card-title">{{ t('config.mountSource') }}</span>
              <span class="card-desc">{{ t('config.mountSourceDesc') }}</span>
            </div>
          </div>
          <div class="input-stack">
            <MiuixInput v-model="mountSource" :label="t('config.mountSource')" placeholder="KSU" single-line class="full-width-field" />
          </div>
        </MiuixCard>
      </section>

      <section class="config-group">
        <MiuixCard class="ex-card">
          <div class="card-header">
            <div class="card-icon">
              <MiuixIcon :icon="Layers" />
            </div>
            <div class="card-text">
              <span class="card-title">{{ t('config.partitions') }}</span>
              <span class="card-desc">{{ t('config.partitionsDesc') }}</span>
            </div>
          </div>
          <div class="chip-list">
            <RemoveableLabel
              v-for="(p, index) in configStore.config.partitions"
              :key="index"
              :text="p"
              @remove="removePartition(index)"
            />
          </div>
          <div class="input-row">
            <MiuixInput v-model="partition" label="e.g. product, system_ext..." single-line class="grow-input" />
            <CustomIconButton v-if="partition" :icon="Add" :size="24" @click="handle_add_partition" />
          </div>
        </MiuixCard>
      </section>

      <section class="config-group">
        <MiuixCard class="ex-card">
          <div class="card-header">
            <div class="card-icon">
              <MiuixIcon :icon="Delete" />
            </div>
            <div class="card-text">
              <span class="card-title">{{ t('config.ignoreList') }}</span>
              <span class="card-desc">{{ t('config.ignoreListDesc') }}</span>
            </div>
          </div>
          <div class="chip-list">
            <RemoveableLabel
              v-for="(path, index) in configStore.config.ignoreList"
              :key="index"
              :text="path"
              @remove="removeIgnorepath(index)"
            />
          </div>
          <div class="input-row">
            <MiuixInput v-model="ignorepath" label="/data/adb/modules/..." single-line class="grow-input" />
            <CustomIconButton v-if="ignorepath" :icon="Add" :size="24" @click="handle_add_ignorepath" />
          </div>
        </MiuixCard>
      </section>

      <section class="config-group">
        <MiuixCard class="ex-card">
          <div class="card-header">
            <div class="card-icon">
              <MiuixIcon :icon="MoveFile" />
            </div>
            <div class="card-text">
              <span class="card-title">{{ t('config.customMounts') }}</span>
              <span class="card-desc">{{ t('config.customMountsDesc') }}</span>
            </div>
          </div>
          <div class="custom-mount-list">
            <BindCard
              v-for="(mount, index) in configStore.config.customMounts"
              :key="index"
              :source="mount.source"
              :target="mount.target"
              :source-label="t('config.customMountSource')"
              :target-label="t('config.customMountTarget')"
              @edit="openEditCustomMountDialog(index)"
            />
          </div>
          <button class="add-custom-mount" @click="openAddCustomMountDialog" type="button">
            <MiuixIcon :icon="Add" />
          </button>
        </MiuixCard>
      </section>

      <section class="config-group">
        <div class="options-grid">
          <button
            class="option-tile"
            :class="{ active: configStore.config.umount }"
            @click="toggleBool('umount')"
            type="button"
          >
            <div class="tile-top">
              <div class="tile-icon">
                <MiuixIcon :icon="Link" />
              </div>
            </div>
            <div class="tile-bottom">
              <span class="tile-label">{{ t('config.umountLabel') }}</span>
              <span class="card-desc">{{ configStore.config.umount ? t('config.umountOn') : t('config.umountOff') }}</span>
            </div>
          </button>
        </div>
      </section>

      <section class="config-group">
        <MiuixCard class="ex-card">
          <div class="bottom-actions">
            <CustomIconButton
              :icon="Refresh"
              :size="24"
              :disabled="configStore.loading"
              @click="reload"
            />
            <div class="spacer" />
            <MiuixButton @click="resetConfig">{{ t('config.reset') }}</MiuixButton>
            <MiuixButton
              type="primary"
              :disabled="configStore.saving || !isDirty"
              @click="saveConfig"
            >
              <MiuixIcon :icon="Ok" :size="18" />
              {{ configStore.saving ? t('common.saving') : t('config.save') }}
            </MiuixButton>
          </div>
        </MiuixCard>
      </section>
    </div>
  </div>
</template>

<style scoped>
.page {
  padding-bottom: 16px;
}

.ex-card {
  margin: 0 12px 12px;
}

.card-header {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 16px;
  border-bottom: 1px solid var(--m-color-outline-variant);
}

.card-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  border-radius: 12px;
  background: var(--m-color-surface-container-highest);
}

.card-text {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.card-title {
  font-size: var(--m-text-title4-size);
  font-weight: 500;
  color: var(--m-color-on-background);
}

.card-desc {
  font-size: var(--m-text-body2-size);
  color: var(--m-color-on-surface-secondary);
}

.input-stack {
  padding: 16px;
}

.full-width-field {
  width: 100%;
}

.chip-list {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  padding: 16px;
}

.chip-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  background: var(--m-color-surface-container-highest);
  border-radius: 16px;
}

.chip-text {
  font-size: var(--m-text-body2-size);
}

.input-row {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 0 16px 16px;
}

.grow-input {
  flex: 1;
}

.custom-mount-list {
  padding: 16px;
}

.custom-mount-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px;
  background: var(--m-color-surface-container-highest);
  border-radius: 12px;
  margin-bottom: 8px;
}

.custom-mount-row:last-child {
  margin-bottom: 0;
}

.custom-mount-row-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.custom-mount-meta {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.custom-mount-label {
  font-size: var(--m-text-caption-size);
  color: var(--m-color-on-surface-tertiary);
}

.custom-mount-value {
  font-size: var(--m-text-body1-size);
  color: var(--m-color-on-background);
}

.add-custom-mount {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 48px;
  height: 48px;
  margin: 0 16px 16px auto;
  border: none;
  border-radius: 50%;
  background: var(--m-color-primary);
  color: var(--m-color-on-primary);
  cursor: pointer;
}

.options-grid {
  display: flex;
  gap: 12px;
  padding: 0 12px;
}

.option-tile {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 20px;
  border: none;
  border-radius: 16px;
  background: var(--m-color-surface-container-low);
  color: var(--m-color-on-surface-variant);
  cursor: pointer;
  transition: all 120ms ease;
}

.option-tile.active {
  background: var(--m-color-surface-container-high);
  color: var(--m-color-primary);
}

.tile-top {
  display: flex;
  align-items: center;
  justify-content: center;
  margin-bottom: 12px;
}

.tile-icon {
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 14px;
  background: var(--m-color-surface-container-highest);
}

.option-tile.active .tile-icon {
  background: var(--m-color-primary-container);
}

.tile-bottom {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.tile-label {
  font-size: var(--m-text-title4-size);
  font-weight: 500;
}

.bottom-actions {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
}

.spacer {
  flex: 1;
}

.lang-action {
  padding: 12px 16px;
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
}
</style>