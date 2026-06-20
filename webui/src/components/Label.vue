<!--

    Copyright (C) 2026 meta-magic_mount-rs developers
    SPDX-License-Identifier: Apache-2.0

-->
<script setup lang="ts">
import { MiuixText } from "miuix-vue";

interface Props {
  bgColor?: string;
  textColor?: string;
  borderRadius?: string;
  padding?: string;
}

withDefaults(defineProps<Props>(), {
  bgColor: "var(--m-color-secondary-container)",
  textColor: "var(--m-color-on-secondary-container)",
  borderRadius: "16px",
  padding: "4px 8px",
});
</script>

<template>
  <div
    class="chip-item"
    :style="{
      backgroundColor: bgColor,
      color: textColor,
      borderRadius: borderRadius,
      padding: padding,
    }"
  >
    <MiuixText type="body2" weight="normal" size="6"><slot /></MiuixText>
  </div>
</template>

<style scoped>
.chip-item {
  display: inline-flex;
  align-items: center;
  margin-right: 8px;
  margin-bottom: 8px;
}
</style>
