/*
 * Copyright (C) 2026 meta-magic_mount-rs developers
 * SPDX-License-Identifier: GPL-v3
 */

import { computed, type ComputedRef, type Component } from "vue";
import { uiStore } from "../stores/uiStore";

export function useStyleComponent(components: {
  miuix: Component;
  md3: Component;
}): ComputedRef<Component> {
  return computed(() => {
    const style = uiStore.uiStyle;
    return components[style] || components.miuix;
  });
}
