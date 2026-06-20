/*
 * Copyright (C) 2026 meta-magic_mount-rs developers
 * SPDX-License-Identifier: Apache-2.0
 */

import { For } from "solid-js";

import { uiStore } from "../lib/stores/uiStore";

import "./Toast.css";

export default () => (
  <div class="toast-container">
    <For each={uiStore.toasts}>
      {(toast) => (
        <div class={`toast toast-${toast.type}`} role="alert">
          <span>{toast.text}</span>
        </div>
      )}
    </For>
  </div>
);
