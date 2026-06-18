<!--

    Copyright (C) 2026 Tools-cx-app <localhost.hutao@gmail.com>
    SPDX-License-Identifier: Apache-2.0

-->
<script setup lang="ts">
import { computed } from "vue";

interface ExtendedIconWeightData {
  vw: number;
  vh: number;
  d: string;
}

type MiuixIconWeight = "light" | "normal" | "regular" | "medium" | "demibold";

interface ExtendedIcon {
  name: string;
  light: ExtendedIconWeightData;
  normal: ExtendedIconWeightData;
  regular: ExtendedIconWeightData;
  medium: ExtendedIconWeightData;
  demibold: ExtendedIconWeightData;
}

interface Props {
  disabled?: boolean;
  holdDown?: boolean;
  size?: number;
  color?: string;
  icon?: ExtendedIcon | ExtendedIconWeightData;
  weight?: MiuixIconWeight;
}

const props = withDefaults(defineProps<Props>(), {
  disabled: false,
  holdDown: false,
  size: 24,
  weight: "regular",
});

const emit = defineEmits<{
  click: [event: MouseEvent];
}>();

const iconStyle = computed(() => ({
  width: `${props.size}px`,
  height: `${props.size}px`,
  color: props.color && props.color !== "none" ? props.color : undefined,
}));

const glyph = computed<ExtendedIconWeightData | null>(() => {
  const i = props.icon;
  if (!i) return null;
  return "regular" in i ? i[props.weight] : i;
});

function onClick(event: MouseEvent): void {
  if (props.disabled) return;
  emit("click", event);
}
</script>

<template>
  <button
    type="button"
    class="m-icon-button"
    :class="{
      'm-icon-button--disabled': props.disabled,
      'm-icon-button--hold-down': props.holdDown,
      'm-icon--no-tint': props.color === 'none',
    }"
    :disabled="props.disabled"
    @click="onClick"
  >
    <span class="m-icon" :style="iconStyle">
      <svg
        v-if="glyph"
        :viewBox="`0 0 ${glyph.vw} ${glyph.vh}`"
        fill="none"
        aria-hidden="true"
      >
        <path
          :d="glyph.d"
          :transform="`matrix(1 0 0 -1 0 ${glyph.vh})`"
          fill="currentColor"
          fill-rule="nonzero"
          clip-rule="nonzero"
        />
      </svg>
      <slot v-else />
    </span>
  </button>
</template>

<style lang="scss">
.m-icon-button {
  position: relative;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: var(--m-icon-button-min-width, 40px);
  min-height: var(--m-icon-button-min-height, 40px);
  padding: 0;
  border: 0;
  border-radius: var(--m-icon-button-radius, 40px);
  background: var(--m-icon-button-bg, transparent);
  color: var(--m-color-on-background);
  appearance: none;
  cursor: pointer;
  user-select: none;
  -webkit-tap-highlight-color: transparent;
  overflow: hidden;

  &::after {
    content: "";
    position: absolute;
    inset: 0;
    border-radius: inherit;
    background: var(--m-color-on-background);
    opacity: 0;
    transition: opacity 120ms linear;
    pointer-events: none;
  }

  &:focus-visible::after {
    opacity: 0.08;
  }
  &:active::after {
    opacity: 0.1;
  }
  &:focus-visible:active::after {
    opacity: 0.18;
  }

  &--hold-down::after {
    opacity: 0.1;
  }
  &--hold-down:focus-visible::after {
    opacity: 0.18;
  }

  @media (hover: hover) {
    &:hover::after {
      opacity: 0.06;
    }
    &:hover:focus-visible::after {
      opacity: 0.14;
    }
    &:hover:active::after {
      opacity: 0.16;
    }
    &:hover:focus-visible:active::after {
      opacity: 0.24;
    }
    &--hold-down:hover::after {
      opacity: 0.16;
    }
    &--hold-down:hover:focus-visible::after {
      opacity: 0.24;
    }
  }

  &--disabled {
    cursor: not-allowed;

    &::after {
      display: none;
    }
  }
}

.m-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  flex: none;
  color: currentColor;

  svg {
    width: 100%;
    height: 100%;
    display: block;
  }

  &:not(.m-icon--no-tint) svg {
    fill: currentColor;
  }
}
</style>

export default { name: 'CustomIconButton', };
