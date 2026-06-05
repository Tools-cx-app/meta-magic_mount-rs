/*
 * Copyright (C) 2026 Tools-cx-app <localhost.hutao@gmail.com>
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
