<!--

    Copyright (C) 2026 meta-magic_mount-rs developers
    SPDX-License-Identifier: GPL-v3

-->
<script setup lang="ts">
import { MiuixCard, MiuixBasicComponent, MiuixText } from "miuix-vue";
import {
  RiCheckboxCircleLine,
  RiIndeterminateCircleLine,
  RiCloseCircleLine,
} from "@remixicon/vue";

interface Props {
  status: "running" | "stopped" | "error";
  label: string;
  description?: string;
  summary?: string;
}

const props = withDefaults(defineProps<Props>(), {
  status: "stopped",
  label: "",
  description: "",
  summary: "",
});

const statusConfig = {
  running: {
    bgColor: "var(--m-color-working-status-card)",
    iconColor: "var(--m-color-working-status-icon)",
  },
  stopped: {
    bgColor: "var(--m-color-secondary-container)",
    iconColor: "var(--m-color-on-secondary-container)",
  },
  error: {
    bgColor: "var(--m-color-error-container)",
    iconColor: "var(--m-color-error)",
  },
};
</script>

<template>
  <MiuixCard
    class="status-card"
    :style="{
      '--m-card-color': statusConfig[props.status].bgColor,
    }"
  >
    <MiuixBasicComponent :title="props.label" :summary="description">
      <template #bottom>
        <MiuixText v-if="summary">{{ summary }}</MiuixText>
      </template>
    </MiuixBasicComponent>
    <div class="status-card__icon">
      <RiCheckboxCircleLine
        v-if="props.status === 'running'"
        size="110px"
        :color="statusConfig[props.status].iconColor"
      />
      <RiIndeterminateCircleLine
        v-else-if="props.status === 'stopped'"
        size="110px"
        :color="statusConfig[props.status].iconColor"
      />
      <RiCloseCircleLine
        v-else-if="props.status === 'error'"
        size="110px"
        :color="statusConfig[props.status].iconColor"
      />
    </div>
  </MiuixCard>
</template>

<style>
/* Additional color tokens */
:root,
.m-theme-light {
  --m-color-working-status-card: #dcfce7;
  --m-color-working-status-icon: #22c55e;
}

.m-theme-dark {
  --m-color-working-status-card: #1b3826;
  --m-color-working-status-icon: #38d167;
}

.status-card {
  padding: 0;
  overflow: visible;
  position: relative;
}
.status-card__icon {
  position: absolute;
  bottom: -32px;
  right: -22px;
}
</style>
