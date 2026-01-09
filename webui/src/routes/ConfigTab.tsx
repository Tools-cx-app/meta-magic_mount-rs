/**
 * Copyright 2025 Magic Mount-rs Authors
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import { createEffect, createMemo, createSignal } from "solid-js";

import BottomActions from "../components/BottomActions";
import ChipInput from "../components/ChipInput";
import { ICONS } from "../lib/constants";
import { store } from "../lib/store";
import type { MagicConfig } from "../types";

import "./ConfigTab.css";
import "@material/web/textfield/outlined-text-field.js";
import "@material/web/button/filled-button.js";
import "@material/web/iconbutton/filled-tonal-icon-button.js";
import "@material/web/iconbutton/icon-button.js";
import "@material/web/icon/icon.js";
import "@material/web/ripple/ripple.js";

export default function ConfigTab() {
  const [initialConfigStr, setInitialConfigStr] = createSignal("");

  const isDirty = createMemo(() => {
    if (!initialConfigStr()) {
      return false;
    }

    return JSON.stringify(store.config) !== initialConfigStr();
  });

  createEffect(() => {
    if (
      !store.loading.config &&
      store.config &&
      (!initialConfigStr() ||
        initialConfigStr() === JSON.stringify(store.config))
    ) {
      setInitialConfigStr(JSON.stringify(store.config));
    }
  });

  function save() {
    store.saveConfig().then(() => {
      setInitialConfigStr(JSON.stringify(store.config));
    });
  }

  function reload() {
    store.loadConfig().then(() => {
      setInitialConfigStr(JSON.stringify(store.config));
    });
  }

  function toggle(key: keyof MagicConfig) {
    const current = store.config[key];
    if (typeof current === "boolean") {
      store.setConfig({ ...store.config, [key]: !current });
    }
  }

  function handleInput(key: keyof MagicConfig, value: string) {
    store.setConfig({ ...store.config, [key]: value });
  }

  function handlePartitionsChange(values: string[]) {
    store.setConfig({ ...store.config, partitions: values });
  }

  return (
    <>
      <div class="config-container">
        <section class="config-group">
          <div class="config-card">
            <div class="card-header">
              <div class="card-icon">
                <md-icon>
                  <svg viewBox="0 0 24 24">
                    <path d={ICONS.ksu} />
                  </svg>
                </md-icon>
              </div>
              <div class="card-text">
                <span class="card-title">{store.L.config.mountSource}</span>
                <span class="card-desc">{store.L.config.mountSourceDesc}</span>
              </div>
            </div>

            <div class="input-stack">
              <md-outlined-text-field
                prop:label={store.L.config.mountSource}
                prop:value={store.config.mountsource}
                on:input={(e: Event) =>
                  handleInput(
                    "mountsource",
                    (e.target as HTMLInputElement).value,
                  )
                }
                class="full-width-field"
              >
                <md-icon slot="leading-icon">
                  <svg viewBox="0 0 24 24">
                    <path d={ICONS.ksu} />
                  </svg>
                </md-icon>
              </md-outlined-text-field>
            </div>
          </div>
        </section>

        <section class="config-group">
          <div class="config-card">
            <div class="card-header">
              <div class="card-icon">
                <md-icon>
                  <svg viewBox="0 0 24 24">
                    <path d={ICONS.storage} />
                  </svg>
                </md-icon>
              </div>
              <div class="card-text">
                <span class="card-title">{store.L.config.partitions}</span>
                <span class="card-desc">{store.L.config.partitionsDesc}</span>
              </div>
            </div>
            <div class="p-input">
              <ChipInput
                values={store.config.partitions}
                placeholder="e.g. product, system_ext..."
                onChange={handlePartitionsChange}
              />
            </div>
          </div>
        </section>

        <section class="config-group">
          <div class="options-grid">
            <button
              class={`option-tile clickable tertiary ${store.config.umount ? "active" : ""}`}
              onClick={() => toggle("umount")}
            >
              <md-ripple />
              <div class="tile-top">
                <div class="tile-icon">
                  <md-icon>
                    <svg viewBox="0 0 24 24">
                      <path d={ICONS.anchor} />
                    </svg>
                  </md-icon>
                </div>
              </div>
              <div class="tile-bottom">
                <span class="tile-label">{store.L.config.umountLabel}</span>
              </div>
            </button>

            <button
              class={`option-tile clickable secondary ${store.fixBottomNav ? "active" : ""}`}
              onClick={store.toggleBottomNavFix}
            >
              <md-ripple />
              <div class="tile-top">
                <div class="tile-icon">
                  <md-icon>
                    <svg viewBox="0 0 24 24">
                      <path d="M21 5v14H3V5h18zm0-2H3c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h18c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zM8 17h5v-6H8v6zm0-8h5V7H8v2zM6 17h2V7H6v10zm12-6h-2v6h2v-6zm0-4h-2v2h2V7z" />
                    </svg>
                  </md-icon>
                </div>
              </div>
              <div class="tile-bottom">
                <span class="tile-label">{store.L.config.fixBottomNav}</span>
              </div>
            </button>
          </div>
        </section>
      </div>

      <BottomActions>
        <md-filled-tonal-icon-button
          on:click={reload}
          prop:disabled={store.loading.config}
          prop:title={store.L.config.reload}
        >
          <md-icon>
            <svg viewBox="0 0 24 24">
              <path d={ICONS.refresh} />
            </svg>
          </md-icon>
        </md-filled-tonal-icon-button>

        <div class="spacer" />

        <md-filled-button
          on:click={save}
          prop:disabled={store.saving.config ?? !isDirty()}
        >
          <md-icon slot="icon">
            <svg viewBox="0 0 24 24">
              <path d={ICONS.save} />
            </svg>
          </md-icon>
          {store.saving.config ? store.L.common.saving : store.L.config.save}
        </md-filled-button>
      </BottomActions>
    </>
  );
}
