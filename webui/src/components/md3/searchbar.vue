<!--

    Copyright (C) 2026 meta-magic_mount-rs developers
    SPDX-License-Identifier: GPL-v3

-->
<script setup lang="ts">
import { ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import { ICONS } from "../../lib/constants";

const { t } = useI18n();

const props = defineProps<{
  modelValue?: string;
}>();

const emit = defineEmits<{
  (e: "update:modelValue", value: string): void;
}>();

const searchQuery = ref(props.modelValue ?? "");

watch(
  () => props.modelValue,
  (val) => {
    searchQuery.value = val ?? "";
  },
);

function handleInput(e: Event) {
  const value = (e.target as HTMLInputElement).value;
  searchQuery.value = value;
  emit("update:modelValue", value);
}

function clearSearch() {
  searchQuery.value = "";
  emit("update:modelValue", "");
}
</script>

<template>
  <div class="search-bar">
    <svg class="search-icon" viewBox="0 0 24 24" width="20" height="20">
      <path :d="ICONS.search" />
    </svg>
    <input
      type="text"
      class="search-input"
      :placeholder="t('modules.searchPlaceholder')"
      :value="searchQuery"
      @input="handleInput"
    />
    <button v-if="searchQuery" class="search-clear-btn" @click="clearSearch">
      <svg viewBox="0 0 24 24" width="18" height="18">
        <path :d="ICONS.close" />
      </svg>
    </button>
  </div>
</template>

<style scoped>
.search-bar {
  background-color: var(--md-sys-color-surface-container-high);
  border-radius: 24px;
  height: 48px;
  display: flex;
  align-items: center;
  padding: 0 16px;
  gap: 12px;
  transition: background-color 0.2s;
}

.search-bar:focus-within {
  background-color: var(--md-sys-color-surface-container-highest);
}

.search-icon {
  fill: var(--md-sys-color-on-surface);
  opacity: 0.7;
  flex-shrink: 0;
}

.search-input {
  flex: 1;
  border: none;
  background: transparent;
  height: 100%;
  font-size: 14px;
  color: var(--md-sys-color-on-surface);
  outline: none;
  min-width: 0;
  padding: 0;
}

.search-input::placeholder {
  color: var(--md-sys-color-on-surface-variant);
  opacity: 0.7;
}

.search-clear-btn {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  border: none;
  background-color: var(--md-sys-color-surface-variant);
  color: var(--md-sys-color-on-surface-variant);
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: background-color 0.2s;
  flex-shrink: 0;
}

.search-clear-btn svg {
  fill: currentColor;
}

.search-clear-btn:hover {
  background-color: var(--md-sys-color-surface-container-highest);
}
</style>
