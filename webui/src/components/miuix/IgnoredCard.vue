<!--

    Copyright (C) 2026 meta-magic_mount-rs developers
    SPDX-License-Identifier: GPL-v3

-->
<script setup lang="ts">
import { MiuixText, MiuixIcon, MiuixIconButton } from "miuix-vue";
import { Close } from "miuix-vue/icons";

interface Props {
  path: string;
}

withDefaults(defineProps<Props>(), {});

const emit = defineEmits<{
  remove: [];
}>();
</script>

<template>
  <div class="ignored-path-row">
    <div class="ignored-path-row-content">
      <div class="ignored-path-meta">
        <MiuixText class="ignored-path-value" :size="13">{{ path }}</MiuixText>
      </div>
    </div>
    <MiuixIconButton @click="emit('remove')">
      <MiuixIcon :icon="Close" :size="16" />
    </MiuixIconButton>
  </div>
</template>

<style scoped>
.ignored-path-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px;
  background-color: var(--m-color-divider-line);
  border-radius: 8px;
  margin-bottom: 8px;
}

.ignored-path-row-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.ignored-path-meta {
  display: flex;
  align-items: center;
  gap: 8px;
}

.ignored-path-value {
  font-size: 13px;
  color: var(--m-color-on-surface-tertiary);
  word-break: break-all;
}
</style>
