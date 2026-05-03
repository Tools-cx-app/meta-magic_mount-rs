/*
 * Copyright (C) 2026 Tools-cx-app <localhost.hutao@gmail.com>
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

import { For, Show, createEffect, createMemo, createSignal } from "solid-js";

import BottomActions from "../components/BottomActions";
import ChipInput from "../components/ChipInput";
import { ICONS } from "../lib/constants";
import { configStore } from "../lib/stores/configStore";
import { uiStore } from "../lib/stores/uiStore";
import type { AppConfig, CustomMount } from "../types";

import "@material/web/button/filled-button.js";
import "@material/web/button/text-button.js";
import "@material/web/dialog/dialog.js";
import "@material/web/icon/icon.js";
import "@material/web/iconbutton/filled-tonal-icon-button.js";
import "@material/web/ripple/ripple.js";
import "@material/web/textfield/outlined-text-field.js";
import "./ConfigTab.css";

interface MdDialogElement extends HTMLElement {
  show: () => void;
  close: () => void;
}

export default function ConfigTab() {
  const [initialConfigStr, setInitialConfigStr] = createSignal("");
  const [customMountDraft, setCustomMountDraft] = createSignal<CustomMount>({
    source: "",
    target: "",
  });
  const [editingCustomMountIndex, setEditingCustomMountIndex] = createSignal<
    number | null
  >(null);
  let mountSourceInputRef: any = null;
  let customMountDialogRef: MdDialogElement | undefined;

  const isDirty = createMemo(() => {
    if (!initialConfigStr()) {
      return false;
    }

    return JSON.stringify(configStore.config) !== initialConfigStr();
  });

  createEffect(() => {
    if (
      !configStore.loading &&
      (!initialConfigStr() ||
        initialConfigStr() === JSON.stringify(configStore.config))
    ) {
      setInitialConfigStr(JSON.stringify(configStore.config));
    }
  });

  function updateConfig<K extends keyof AppConfig>(
    key: K,
    value: AppConfig[K],
  ) {
    configStore.setConfig({ ...configStore.config, [key]: value });
  }

  function save() {
    void configStore.saveConfig().then(() => {
      setInitialConfigStr(JSON.stringify(configStore.config));
    });
  }

  function reload() {
    void configStore.loadConfig().then(() => {
      setInitialConfigStr(JSON.stringify(configStore.config));
    });
  }

  function toggleBool(key: keyof AppConfig) {
    const currentValue = configStore.config[key];

    if (typeof currentValue === "boolean") {
      updateConfig(key, !currentValue as AppConfig[typeof key]);
    }
  }

  function updateCustomMountDraft(key: keyof CustomMount, value: string) {
    setCustomMountDraft((prev) => ({ ...prev, [key]: value }));
  }

  function openAddCustomMountDialog() {
    setEditingCustomMountIndex(null);
    setCustomMountDraft({ source: "", target: "" });
    customMountDialogRef?.show();
  }

  function openEditCustomMountDialog(index: number) {
    setEditingCustomMountIndex(index);
    setCustomMountDraft({ ...configStore.config.customMounts[index] });
    customMountDialogRef?.show();
  }

  function closeCustomMountDialog() {
    customMountDialogRef?.close();
  }

  function saveCustomMountDialog() {
    const draft = {
      source: customMountDraft().source.trim(),
      target: customMountDraft().target.trim(),
    };

    if (!draft.source || !draft.target) {
      return;
    }

    if (editingCustomMountIndex() === null) {
      updateConfig("customMounts", [...configStore.config.customMounts, draft]);
    } else {
      updateConfig(
        "customMounts",
        configStore.config.customMounts.map((mount, index) =>
          index === editingCustomMountIndex() ? draft : mount,
        ),
      );
    }

    closeCustomMountDialog();
  }

  function deleteCustomMountDialog() {
    const index = editingCustomMountIndex();

    if (index === null) {
      return;
    }

    updateConfig(
      "customMounts",
      configStore.config.customMounts.filter(
        (_, mountIndex) => mountIndex !== index,
      ),
    );
    closeCustomMountDialog();
  }

  return (
    <>
      <div class="dialog-container">
        <md-dialog ref={customMountDialogRef}>
          <div slot="headline">
            {editingCustomMountIndex() === null
              ? uiStore.L.config.customMountDialogAdd
              : uiStore.L.config.customMountDialogEdit}
          </div>
          <div slot="content" class="custom-mount-dialog-content">
            <div class="custom-mount-dialog-fields">
              <md-outlined-text-field
                label={uiStore.L.config.customMountSource}
                placeholder="/data/adb/modules/test/bin/unit"
                value={customMountDraft().source}
                onInput={(event: InputEvent) =>
                  updateCustomMountDraft(
                    "source",
                    (event.currentTarget as HTMLInputElement).value,
                  )
                }
                class="full-width-field"
              />
              <md-outlined-text-field
                label={uiStore.L.config.customMountTarget}
                placeholder="/product/bin/unit"
                value={customMountDraft().target}
                onInput={(event: InputEvent) =>
                  updateCustomMountDraft(
                    "target",
                    (event.currentTarget as HTMLInputElement).value,
                  )
                }
                class="full-width-field"
              />
            </div>
          </div>
          <div slot="actions">
            <Show when={editingCustomMountIndex() !== null}>
              <md-filled-tonal-icon-button
                onClick={deleteCustomMountDialog}
                title={uiStore.L.config.removeCustomMount}
              >
                <md-icon>
                  <svg viewBox="0 0 24 24">
                    <path d={ICONS.delete} />
                  </svg>
                </md-icon>
              </md-filled-tonal-icon-button>
              <div class="spacer"></div>
            </Show>
            <md-text-button onClick={closeCustomMountDialog}>
              {uiStore.L.common.cancel}
            </md-text-button>
            <md-text-button onClick={saveCustomMountDialog}>
              {uiStore.L.config.customMountDialogSave}
            </md-text-button>
          </div>
        </md-dialog>
      </div>

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
                <span class="card-title">{uiStore.L.config.mountSource}</span>
                <span class="card-desc">
                  {uiStore.L.config.mountSourceDesc}
                </span>
              </div>
            </div>

            <div class="input-stack">
              <md-outlined-text-field
                ref={(el) => (mountSourceInputRef = el)}
                label={uiStore.L.config.mountSource}
                placeholder="KSU"
                value={configStore.config.mountsource}
                onInput={(event: InputEvent) =>
                  updateConfig(
                    "mountsource",
                    (event.currentTarget as HTMLInputElement).value,
                  )
                }
                onFocus={() => {
                  setTimeout(() => {
                    mountSourceInputRef?.scrollIntoView({
                      behavior: "smooth",
                      block: "center",
                    });
                  }, 300);
                }}
                class="full-width-field"
              />
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
                <span class="card-title">{uiStore.L.config.partitions}</span>
                <span class="card-desc">{uiStore.L.config.partitionsDesc}</span>
              </div>
            </div>

            <ChipInput
              values={configStore.config.partitions}
              placeholder="e.g. product, system_ext..."
              onValuesChange={(values) => updateConfig("partitions", values)}
            />
          </div>
        </section>

        <section class="config-group">
          <div class="config-card">
            <div class="card-header">
              <div class="card-icon">
                <md-icon>
                  <svg viewBox="0 0 24 24">
                    <path d={ICONS.delete} />
                  </svg>
                </md-icon>
              </div>
              <div class="card-text">
                <span class="card-title">{uiStore.L.config.ignoreList}</span>
                <span class="card-desc">{uiStore.L.config.ignoreListDesc}</span>
              </div>
            </div>

            <ChipInput
              values={configStore.config.ignoreList}
              placeholder="/data/adb/modules/..."
              onValuesChange={(values) => updateConfig("ignoreList", values)}
            />
          </div>
        </section>

        <section class="config-group">
          <div class="config-card">
            <div class="card-header">
              <div class="card-icon">
                <md-icon>
                  <svg viewBox="0 -960 960 960">
                    <path d={ICONS.mount_path} />
                  </svg>
                </md-icon>
              </div>
              <div class="card-text">
                <span class="card-title">{uiStore.L.config.customMounts}</span>
                <span class="card-desc">
                  {uiStore.L.config.customMountsDesc}
                </span>
              </div>
            </div>

            <div class="custom-mount-list">
              <For each={configStore.config.customMounts}>
                {(mount, index) => (
                  <div class="custom-mount-row">
                    <div class="custom-mount-row-content">
                      <div class="custom-mount-meta">
                        <span class="custom-mount-label">
                          {uiStore.L.config.customMountSource}
                        </span>
                        <span class="custom-mount-value">{mount.source}</span>
                      </div>
                      <div class="custom-mount-meta">
                        <span class="custom-mount-label">
                          {uiStore.L.config.customMountTarget}
                        </span>
                        <span class="custom-mount-value">{mount.target}</span>
                      </div>
                    </div>
                    <md-filled-tonal-icon-button
                      onClick={() => openEditCustomMountDialog(index())}
                      title={uiStore.L.config.editCustomMount}
                    >
                      <md-icon>
                        <svg viewBox="0 0 24 24">
                          <path d={ICONS.settings} />
                        </svg>
                      </md-icon>
                    </md-filled-tonal-icon-button>
                  </div>
                )}
              </For>
            </div>

            <button
              class="add-custom-mount"
              onClick={openAddCustomMountDialog}
              title={uiStore.L.config.addCustomMount}
              type="button"
            >
              <md-ripple />
              <md-icon>
                <svg viewBox="0 0 24 24">
                  <path d={ICONS.add} />
                </svg>
              </md-icon>
            </button>
          </div>
        </section>

        <section class="config-group">
          <div class="options-grid">
            <button
              class={`option-tile clickable tertiary ${configStore.config.umount ? "active" : ""}`}
              onClick={() => toggleBool("umount")}
              type="button"
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
                <span class="tile-label">{uiStore.L.config.umountLabel}</span>
                <span class="card-desc">
                  {configStore.config.umount
                    ? uiStore.L.config.umountOn
                    : uiStore.L.config.umountOff}
                </span>
              </div>
            </button>
          </div>
        </section>
      </div>

      <BottomActions>
        <md-filled-tonal-icon-button
          onClick={reload}
          disabled={configStore.loading}
          title={uiStore.L.config.reload}
        >
          <md-icon>
            <svg viewBox="0 0 24 24">
              <path d={ICONS.refresh} />
            </svg>
          </md-icon>
        </md-filled-tonal-icon-button>

        <div class="spacer" />

        <md-filled-button
          onClick={save}
          disabled={configStore.saving || !isDirty()}
        >
          <md-icon slot="icon">
            <svg viewBox="0 0 24 24">
              <path d={ICONS.save} />
            </svg>
          </md-icon>
          {configStore.saving ? uiStore.L.common.saving : uiStore.L.config.save}
        </md-filled-button>
      </BottomActions>
    </>
  );
}
