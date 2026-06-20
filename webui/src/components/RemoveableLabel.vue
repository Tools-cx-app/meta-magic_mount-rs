<!--

    Copyright (C) 2026 meta-magic_mount-rs developers
    SPDX-License-Identifier: Apache-2.0

-->
<script setup lang="ts">
import { MiuixText, MiuixIcon, MiuixIconButton } from "miuix-vue";
import { Close } from "miuix-vue/icons";

interface Props {
  text: string;
  buttonalign?: string;
}

const props = withDefaults(defineProps<Props>(), {
  buttonalign: "end",
});

const emit = defineEmits<{
  remove: [];
}>();
</script>

<template>
  <div class="chip-item">
    <MiuixIconButton
      v-if="props.buttonalign === 'start'"
      @click="emit('remove')"
    >
      <MiuixIcon :icon="Close" :size="16" />
    </MiuixIconButton>
    <MiuixText size="13">{{ text }}</MiuixText>
    <MiuixIconButton v-if="props.buttonalign === 'end'" @click="emit('remove')">
      <MiuixIcon :icon="Close" :size="16" />
    </MiuixIconButton>
  </div>
</template>

<style scoped>
.chip-item {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  background-color: var(--m-color-secondary-container);
  border-radius: 16px;
  margin-right: 8px;
  margin-bottom: 8px;
}
</style>
