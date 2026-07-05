<!--

    Copyright (C) 2026 meta-magic_mount-rs developers
    SPDX-License-Identifier: GPL-v3

-->
<script setup lang="ts">
import { ref, computed, watch, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import BottomActions from "../components/md3/BottomActions.vue";
import ChipInput from "../components/md3/ChipInput.vue";
import { ICONS } from "../lib/constants";
import { configStore } from "../lib/stores/configStore";
import { uiStore } from "../lib/stores/uiStore";
import type { AppConfig, CustomMount } from "../lib/types";

const { t } = useI18n();

const currentLocale = ref(uiStore.lang);
const currentUiStyle = ref<"miuix" | "md3">(uiStore.uiStyle);

const initialConfigStr = ref("");
const customMountDraft = ref<CustomMount>({ source: "", target: "" });
const editingCustomMountIndex = ref<number | null>(null);
const showCustomMountDialog = ref(false);

const isDirty = computed(() => {
  if (!initialConfigStr.value) return false;
  return JSON.stringify(configStore.config) !== initialConfigStr.value;
});

watch(
  () => configStore.config,
  (newConfig) => {
    if (
      !configStore.loading &&
      (!initialConfigStr.value ||
        initialConfigStr.value === JSON.stringify(newConfig))
    ) {
      initialConfigStr.value = JSON.stringify(newConfig);
    }
  },
  { deep: true },
);

function updateConfig<K extends keyof AppConfig>(key: K, value: AppConfig[K]) {
  configStore.setConfig({ ...configStore.config, [key]: value });
}

async function save() {
  const success = await configStore.saveConfig();
  if (success) {
    initialConfigStr.value = JSON.stringify(configStore.config);
  }
}

function reload() {
  void configStore.loadConfig().then(() => {
    initialConfigStr.value = JSON.stringify(configStore.config);
  });
}

function toggleBool(key: keyof AppConfig) {
  const currentValue = configStore.config[key];
  if (typeof currentValue === "boolean") {
    updateConfig(key, !currentValue as AppConfig[typeof key]);
  }
}

function openAddCustomMountDialog() {
  editingCustomMountIndex.value = null;
  customMountDraft.value = { source: "", target: "" };
  showCustomMountDialog.value = true;
}

function openEditCustomMountDialog(index: number) {
  editingCustomMountIndex.value = index;
  customMountDraft.value = { ...configStore.config.customMounts[index] };
  showCustomMountDialog.value = true;
}

function closeCustomMountDialog() {
  showCustomMountDialog.value = false;
}

function saveCustomMountDialog() {
  const draft = {
    source: customMountDraft.value.source.trim(),
    target: customMountDraft.value.target.trim(),
  };

  if (!draft.source || !draft.target) return;

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

function deleteCustomMountDialog() {
  const index = editingCustomMountIndex.value;
  if (index === null) return;

  updateConfig(
    "customMounts",
    configStore.config.customMounts.filter(
      (_, mountIndex) => mountIndex !== index,
    ),
  );
  closeCustomMountDialog();
}

async function changeLocale() {
  await uiStore.setLang(currentLocale.value);
}

function changeUiStyle() {
  uiStore.setUiStyle(currentUiStyle.value);
}

onMounted(() => {
  void uiStore.fetchAvailableLanguages();
  void configStore.loadConfig().then(() => {
    initialConfigStr.value = JSON.stringify(configStore.config);
  });
});
</script>

<template>
  <div class="md3-page config-container">
    <div
      v-if="showCustomMountDialog"
      class="dialog-overlay"
      @click.self="closeCustomMountDialog"
    >
      <div class="dialog-content">
        <div class="dialog-headline">
          {{
            editingCustomMountIndex === null
              ? t("config.customMountDialogAdd")
              : t("config.customMountDialogEdit")
          }}
        </div>
        <div class="custom-mount-dialog-content">
          <div class="custom-mount-dialog-fields">
            <div class="text-field-wrapper">
              <label class="text-field-label">
                {{ t("config.customMountSource") }}
              </label>
              <input
                class="text-field-input"
                :placeholder="'/data/adb/modules/test/bin/unit'"
                v-model="customMountDraft.source"
              />
            </div>
            <div class="text-field-wrapper">
              <label class="text-field-label">
                {{ t("config.customMountTarget") }}
              </label>
              <input
                class="text-field-input"
                :placeholder="'/product/bin/unit'"
                v-model="customMountDraft.target"
              />
            </div>
          </div>
        </div>
        <div class="dialog-actions">
          <template v-if="editingCustomMountIndex !== null">
            <button
              class="dialog-btn-icon"
              @click="deleteCustomMountDialog"
              :title="t('config.removeCustomMount')"
            >
              <svg viewBox="0 0 24 24" width="24" height="24">
                <path :d="ICONS.delete" />
              </svg>
            </button>
            <div class="spacer" />
          </template>
          <button class="dialog-btn-text" @click="closeCustomMountDialog">
            {{ t("common.cancel") }}
          </button>
          <button class="dialog-btn-text" @click="saveCustomMountDialog">
            {{ t("config.customMountDialogSave") }}
          </button>
        </div>
      </div>
    </div>

    <section class="config-group">
      <div class="config-card">
        <div class="card-header">
          <div class="card-icon">
            <svg viewBox="0 0 24 24" width="24" height="24">
              <path :d="ICONS.translate" />
            </svg>
          </div>
          <div class="card-text">
            <span class="card-title">{{ t("common.language") }}</span>
          </div>
        </div>

        <div class="text-field-wrapper">
          <label class="text-field-label">{{ t("common.language") }}</label>
          <select
            class="text-field-input"
            v-model="currentLocale"
            @change="changeLocale"
          >
            <option
              v-for="lang in uiStore.availableLanguages"
              :key="lang.code"
              :value="lang.code"
            >
              {{ lang.display }}
            </option>
          </select>
        </div>
      </div>
    </section>

    <section class="config-group">
      <div class="config-card">
        <div class="card-header">
          <div class="card-icon">
            <svg viewBox="0 -960 960 960" width="24" height="24">
              <path :d="ICONS.paint" />
            </svg>
          </div>
          <div class="card-text">
            <span class="card-title">{{ t("config.uiStyle") }}</span>
          </div>
        </div>

        <div class="options-grid">
          <button
            class="option-tile clickable secondary"
            :class="{ active: currentUiStyle === 'miuix' }"
            @click="
              currentUiStyle = 'miuix';
              changeUiStyle();
            "
          >
            <div class="tile-top">
              <div class="tile-icon">
                <svg viewBox="0 0 1200 1200" width="24" height="24">
                  <path :d="ICONS.miuix" />
                </svg>
              </div>
            </div>
            <div class="tile-bottom">
              <span class="tile-label">MiuiX</span>
            </div>
          </button>
          <button
            class="option-tile clickable secondary"
            :class="{ active: currentUiStyle === 'md3' }"
            @click="
              currentUiStyle = 'md3';
              changeUiStyle();
            "
          >
            <div class="tile-top">
              <div class="tile-icon">
                <svg viewBox="0 -960 960 960" width="24" height="24">
                  <path :d="ICONS.android" />
                </svg>
              </div>
            </div>
            <div class="tile-bottom">
              <span class="tile-label">Material Design 3</span>
            </div>
          </button>
        </div>
      </div>
    </section>

    <section class="config-group">
      <div class="config-card">
        <div class="card-header">
          <div class="card-icon">
            <svg viewBox="0 0 24 24" width="24" height="24">
              <path :d="ICONS.ksu" />
            </svg>
          </div>
          <div class="card-text">
            <span class="card-title">{{ t("config.mountSource") }}</span>
            <span class="card-desc">{{ t("config.mountSourceDesc") }}</span>
          </div>
        </div>

        <div class="input-stack">
          <div class="text-field-wrapper">
            <label class="text-field-label">
              {{ t("config.mountSource") }}
            </label>
            <input
              class="text-field-input"
              placeholder="KSU"
              :value="configStore.config.mountsource"
              @input="
                (e) =>
                  updateConfig(
                    'mountsource',
                    (e.target as HTMLInputElement).value,
                  )
              "
            />
          </div>
        </div>
      </div>
    </section>

    <section class="config-group">
      <div class="config-card">
        <div class="card-header">
          <div class="card-icon">
            <svg viewBox="0 0 24 24" width="24" height="24">
              <path :d="ICONS.storage" />
            </svg>
          </div>
          <div class="card-text">
            <span class="card-title">{{ t("config.partitions") }}</span>
            <span class="card-desc">{{ t("config.partitionsDesc") }}</span>
          </div>
        </div>

        <ChipInput
          :values="configStore.config.partitions"
          placeholder="e.g. product, system_ext..."
          @update:values="(values) => updateConfig('partitions', values)"
        />
      </div>
    </section>

    <section class="config-group">
      <div class="config-card">
        <div class="card-header">
          <div class="card-icon">
            <svg viewBox="0 0 24 24" width="24" height="24">
              <path :d="ICONS.delete" />
            </svg>
          </div>
          <div class="card-text">
            <span class="card-title">{{ t("config.ignoreList") }}</span>
            <span class="card-desc">{{ t("config.ignoreListDesc") }}</span>
          </div>
        </div>

        <ChipInput
          :values="configStore.config.ignoreList"
          placeholder="/data/adb/modules/..."
          @update:values="(values) => updateConfig('ignoreList', values)"
        />
      </div>
    </section>

    <section class="config-group">
      <div class="config-card">
        <div class="card-header">
          <div class="card-icon">
            <svg viewBox="0 -960 960 960" width="24" height="24">
              <path :d="ICONS.mount_path" />
            </svg>
          </div>
          <div class="card-text">
            <span class="card-title">{{ t("config.customMounts") }}</span>
            <span class="card-desc">{{ t("config.customMountsDesc") }}</span>
          </div>
        </div>

        <div class="custom-mount-list">
          <div
            v-for="(mount, index) in configStore.config.customMounts"
            :key="index"
            class="custom-mount-row"
          >
            <div class="custom-mount-row-content">
              <div class="custom-mount-meta">
                <span class="custom-mount-label">
                  {{ t("config.customMountSource") }}
                </span>
                <span class="custom-mount-value">{{ mount.source }}</span>
              </div>
              <div class="custom-mount-meta">
                <span class="custom-mount-label">
                  {{ t("config.customMountTarget") }}
                </span>
                <span class="custom-mount-value">{{ mount.target }}</span>
              </div>
            </div>
            <button
              class="custom-mount-edit-btn"
              @click="openEditCustomMountDialog(index)"
              :title="t('config.editCustomMount')"
            >
              <svg viewBox="0 0 24 24" width="20" height="20">
                <path :d="ICONS.settings" />
              </svg>
            </button>
          </div>
        </div>

        <button
          class="add-custom-mount"
          @click="openAddCustomMountDialog"
          :title="t('config.addCustomMount')"
        >
          <svg viewBox="0 0 24 24" width="20" height="20">
            <path :d="ICONS.add" />
          </svg>
        </button>
      </div>
    </section>

    <section class="config-group">
      <div class="options-grid">
        <button
          class="option-tile clickable tertiary"
          :class="{ active: configStore.config.umount }"
          @click="toggleBool('umount')"
        >
          <div class="tile-top">
            <div class="tile-icon">
              <svg viewBox="0 0 24 24" width="24" height="24">
                <path :d="ICONS.anchor" />
              </svg>
            </div>
          </div>
          <div class="tile-bottom">
            <span class="tile-label">{{ t("config.umountLabel") }}</span>
            <span class="card-desc">
              {{
                configStore.config.umount
                  ? t("config.umountOn")
                  : t("config.umountOff")
              }}
            </span>
          </div>
        </button>
      </div>
    </section>

    <BottomActions>
      <button
        class="action-btn-icon"
        @click="reload"
        :disabled="configStore.loading"
        :title="t('config.reload')"
      >
        <svg viewBox="0 0 24 24" width="24" height="24">
          <path :d="ICONS.refresh" />
        </svg>
      </button>

      <div class="spacer" />

      <button
        class="action-btn-filled"
        @click="save"
        :disabled="configStore.saving || !isDirty"
      >
        <svg viewBox="0 0 24 24" width="20" height="20">
          <path :d="ICONS.save" />
        </svg>
        <span>
          {{ configStore.saving ? t("common.saving") : t("config.save") }}
        </span>
      </button>
    </BottomActions>
  </div>
</template>

<style scoped>
.config-container {
  display: flex;
  flex-direction: column;
  gap: 24px;
  padding-bottom: 24px;
}

.config-group {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.config-card {
  background-color: var(--md-sys-color-surface-container);
  border-radius: 20px;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.card-header {
  display: flex;
  align-items: center;
  gap: 12px;
}

.card-icon {
  width: 40px;
  height: 40px;
  flex-shrink: 0;
  border-radius: 12px;
  background-color: var(--md-sys-color-secondary-container);
  color: var(--md-sys-color-on-secondary-container);
  display: flex;
  align-items: center;
  justify-content: center;
}

.card-icon svg {
  fill: currentColor;
}

.card-text {
  display: flex;
  flex-direction: column;
}

.card-title {
  font-size: 16px;
  font-weight: 500;
  color: var(--md-sys-color-on-surface);
}

.card-desc {
  font-size: 12px;
  color: var(--md-sys-color-on-surface-variant);
  white-space: pre-line;
}

.input-stack {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.text-field-wrapper {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.text-field-label {
  font-size: 12px;
  font-weight: 500;
  color: var(--md-sys-color-on-surface-variant);
  padding-left: 4px;
}

.text-field-input {
  width: 100%;
  padding: 12px 16px;
  border: 1px solid var(--md-sys-color-outline);
  border-radius: 8px;
  background-color: var(--md-sys-color-surface-container-high);
  color: var(--md-sys-color-on-surface);
  font-size: 16px;
  outline: none;
  transition: border-color 0.2s;
  -webkit-appearance: none;
  appearance: none;
  cursor: pointer;
}

.text-field-input:focus {
  border-color: var(--md-sys-color-primary);
}

.text-field-input::placeholder {
  color: var(--md-sys-color-on-surface-variant);
  opacity: 0.6;
}

.text-field-input:-webkit-autofill {
  -webkit-box-shadow: 0 0 0 100px var(--md-sys-color-surface-container-high)
    inset;
  -webkit-text-fill-color: var(--md-sys-color-on-surface);
}

.custom-mount-list {
  display: flex;
  flex-direction: column;
  gap: 2px;
  align-items: flex-start;
  border-radius: 20px;
  overflow: clip;
}

.custom-mount-row {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: 12px;
  align-items: center;
  width: 100%;
  padding: 16px;
  border-radius: 4px;
  background-color: var(--md-sys-color-surface-container-high);
}

.custom-mount-row-content {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
}

.custom-mount-meta {
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.custom-mount-label {
  font-size: 11px;
  color: var(--md-sys-color-on-surface-variant);
  user-select: none;
}

.custom-mount-value {
  font-size: 14px;
  color: var(--md-sys-color-on-surface);
  line-height: 1.4;
  overflow-wrap: anywhere;
  user-select: none;
}

.custom-mount-edit-btn {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  border: none;
  background-color: var(--md-sys-color-secondary-container);
  color: var(--md-sys-color-on-secondary-container);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: background-color 0.2s;
}

.custom-mount-edit-btn svg {
  fill: currentColor;
}

.custom-mount-edit-btn:hover {
  background-color: var(--md-sys-color-secondary-container-high);
}

.add-custom-mount {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 100%;
  height: 48px;
  padding: 0;
  border: 1px dashed var(--md-sys-color-outline);
  border-radius: 8px;
  background-color: transparent;
  color: var(--md-sys-color-primary);
  cursor: pointer;
  transition: border-color 0.2s;
}

.add-custom-mount svg {
  fill: currentColor;
}

.add-custom-mount:hover {
  border-color: var(--md-sys-color-primary);
}

.options-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

.option-tile {
  background-color: var(--md-sys-color-surface-container);
  border-radius: 20px;
  padding: 16px;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  gap: 12px;
  transition:
    background-color 0.2s cubic-bezier(0.2, 0, 0, 1),
    transform 0.2s;
  border: none;
  text-align: left;
  cursor: pointer;
}

.option-tile:active {
  transform: scale(0.98);
}

.tile-top {
  display: flex;
  align-items: flex-start;
  pointer-events: none;
}

.tile-icon {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  transition:
    background-color 0.2s,
    color 0.2s;
  background-color: var(--md-sys-color-surface-container-high);
  color: var(--md-sys-color-on-surface-variant);
}

.tile-icon svg {
  fill: currentColor;
}

.tile-bottom {
  display: flex;
  flex-direction: column;
  gap: 2px;
  width: 100%;
  pointer-events: none;
}

.tile-label {
  font-size: 15px;
  font-weight: 500;
  color: var(--md-sys-color-on-surface);
  line-height: 1.2;
  white-space: pre-line;
}

.option-tile.tertiary.active {
  background-color: var(--md-sys-color-tertiary-container);
}

.option-tile.tertiary.active .tile-icon {
  background-color: var(--md-sys-color-on-tertiary-container);
  color: var(--md-sys-color-tertiary-container);
}

.option-tile.tertiary.active .tile-label {
  color: var(--md-sys-color-on-tertiary-container);
}

.option-tile.secondary.active {
  background-color: var(--md-sys-color-secondary-container);
}

.option-tile.secondary.active .tile-icon {
  background-color: var(--md-sys-color-on-secondary-container);
  color: var(--md-sys-color-secondary-container);
}

.option-tile.secondary.active .tile-label {
  color: var(--md-sys-color-on-secondary-container);
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
  max-width: 400px;
}

.dialog-headline {
  font-size: 20px;
  font-weight: 500;
  color: var(--md-sys-color-on-surface);
  margin-bottom: 16px;
}

.custom-mount-dialog-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.custom-mount-dialog-fields {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  margin-top: 24px;
  align-items: center;
}

.dialog-btn-icon {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  border: none;
  background-color: var(--md-sys-color-error-container);
  color: var(--md-sys-color-on-error-container);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: background-color 0.2s;
}

.dialog-btn-icon svg {
  fill: currentColor;
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

.spacer {
  flex: 1;
}

.action-btn-icon {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  border: none;
  background-color: var(--md-sys-color-secondary-container);
  color: var(--md-sys-color-on-secondary-container);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: background-color 0.2s;
}

.action-btn-icon svg {
  fill: currentColor;
}

.action-btn-icon:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.action-btn-filled {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px 20px;
  border-radius: 16px;
  border: none;
  background-color: var(--md-sys-color-primary);
  color: var(--md-sys-color-on-primary);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  transition: background-color 0.2s;
}

.action-btn-filled svg {
  fill: currentColor;
}

.action-btn-filled:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
