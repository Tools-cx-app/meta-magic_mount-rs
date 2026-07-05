<!--

    Copyright (C) 2026 meta-magic_mount-rs developers
    SPDX-License-Identifier: GPL-v3

-->
<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";

const isActivePage = ref(true);

onMounted(() => {
  isActivePage.value = true;
});

onUnmounted(() => {
  isActivePage.value = false;
});
</script>

<template>
  <div class="bottom-actions-root" :class="{ 'is-active': isActivePage }">
    <slot />
  </div>
</template>

<style scoped>
.bottom-actions-root {
  position: fixed;
  left: 0;
  right: 0;
  bottom: calc(
    var(--bottom-nav-height, 64px) + var(--bottom-inset, 0px) + 16px
  );
  display: flex;
  align-items: center;
  padding: 0 16px;
  gap: 16px;
  z-index: 90;
  opacity: 0;
  pointer-events: none;
  will-change: opacity, transform;
  transition:
    opacity 0.2s ease,
    transform 0.24s cubic-bezier(0.2, 1, 0.2, 1);
}

.bottom-actions-root.is-active {
  opacity: 1;
}

.bottom-actions-root > * {
  pointer-events: auto;
}

.bottom-actions-root > .spacer {
  flex: 1;
  pointer-events: none;
}
</style>
