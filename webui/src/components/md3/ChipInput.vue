<!--

    Copyright (C) 2026 meta-magic_mount-rs developers
    SPDX-License-Identifier: GPL-v3

-->
<script setup lang="ts">
import { ref } from "vue";
import { ICONS } from "../../lib/constants";

const props = defineProps<{
  values: string[];
  placeholder?: string;
}>();

const emit = defineEmits<{
  (e: "update:values", values: string[]): void;
}>();

const inputValue = ref("");

function handleAdd() {
  const value = inputValue.value.trim();
  if (value && !props.values.includes(value)) {
    emit("update:values", [...props.values, value]);
    inputValue.value = "";
  }
}

function handleRemove(index: number) {
  emit(
    "update:values",
    props.values.filter((_, i) => i !== index),
  );
}

function handleKeydown(e: KeyboardEvent) {
  if (e.key === "Enter") {
    e.preventDefault();
    handleAdd();
  }
}
</script>

<template>
  <div class="chip-input-wrapper">
    <div v-if="values.length > 0" class="chip-set">
      <div v-for="(value, index) in values" :key="index" class="chip-item">
        <span class="chip-text">{{ value }}</span>
        <button class="chip-remove" @click="handleRemove(index)">
          <svg viewBox="0 0 24 24" width="28" height="28">
            <path :d="ICONS.delete" />
          </svg>
        </button>
      </div>
    </div>
    <div class="input-row">
      <input
        v-model="inputValue"
        :placeholder="placeholder"
        class="chip-input-field"
        @keydown="handleKeydown"
      />
      <button class="add-btn" @click="handleAdd">
        <svg viewBox="0 0 24 24" width="20" height="20">
          <path :d="ICONS.add" />
        </svg>
      </button>
    </div>
  </div>
</template>

<style scoped>
.chip-input-wrapper {
  display: flex;
  flex-direction: column;
  padding: 12px;
  border: 1px solid var(--md-sys-color-outline);
  border-radius: 8px;
}

.chip-set {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 8px;
}

.chip-item {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  background-color: var(--md-sys-color-secondary-container);
  color: var(--md-sys-color-on-secondary-container);
  border-radius: 16px;
  font-size: 14px;
}

.chip-remove {
  width: 28px;
  height: 28px;
  border: none;
  background: transparent;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  color: inherit;
}

.input-row {
  display: flex;
  align-items: center;
  gap: 8px;
}

.chip-input-field {
  flex: 1;
  min-width: 80px;
  border: none;
  background: transparent;
  font-size: 16px;
  color: var(--md-sys-color-on-surface);
  outline: none;
  height: 32px;
}

.add-btn {
  width: 32px;
  height: 32px;
  border: none;
  background: transparent;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--md-sys-color-primary);
  cursor: pointer;
}
</style>
