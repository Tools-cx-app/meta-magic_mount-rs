import { createEffect, createMemo, createSignal } from "solid-js";

import BottomActions from "../components/BottomActions";
import ChipInput from "../components/ChipInput";
import { ICONS } from "../lib/constants";
import { configStore } from "../lib/stores/configStore";
import { uiStore } from "../lib/stores/uiStore";
import type { AppConfig } from "../types";

import "./ConfigTab.css";
import "@material/web/textfield/outlined-text-field.js";
import "@material/web/button/filled-button.js";
import "@material/web/iconbutton/filled-tonal-icon-button.js";
import "@material/web/icon/icon.js";
import "@material/web/list/list.js";
import "@material/web/list/list-item.js";
import "@material/web/switch/switch.js";

export default function ConfigTab() {
  const [initialConfigStr, setInitialConfigStr] = createSignal("");

  const isDirty = createMemo(() => {
    if (!initialConfigStr()) {
      return false;
    }

    return JSON.stringify(configStore.config) !== initialConfigStr();
  });

  createEffect(() => {
    if (
      !configStore.loading &&
      configStore.config &&
      (!initialConfigStr() ||
        initialConfigStr() === JSON.stringify(configStore.config))
    ) {
      setInitialConfigStr(JSON.stringify(configStore.config));
    }
  });

  function save() {
    configStore.saveConfig().then(() => {
      setInitialConfigStr(JSON.stringify(configStore.config));
    });
  }

  function reload() {
    configStore.loadConfig().then(() => {
      setInitialConfigStr(JSON.stringify(configStore.config));
    });
  }

  function toggle(key: keyof AppConfig) {
    const current = configStore.config[key];
    if (typeof current === "boolean") {
      configStore.setConfig({ ...configStore.config, [key]: !current });
    }
  }

  function handleInput(key: keyof AppConfig, value: string) {
    configStore.setConfig({ ...configStore.config, [key]: value });
  }

  function handlePartitionsChange(values: string[]) {
    configStore.setConfig({ ...configStore.config, partitions: values });
  }

  function handleIgnoreListChange(values: string[]) {
    configStore.setConfig({ ...configStore.config, ignoreList: values });
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
                <span class="card-title">{uiStore.L.config.mountSource}</span>
                <span class="card-desc">
                  {uiStore.L.config.mountSourceDesc}
                </span>
              </div>
            </div>

            <div class="input-stack">
              <md-outlined-text-field
                prop:label={uiStore.L.config.mountSource}
                prop:value={configStore.config.mountsource}
                on:input={(e: Event) =>
                  handleInput(
                    "mountsource",
                    (e.target as HTMLInputElement).value,
                  )
                }
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
            <div class="p-input">
              <ChipInput
                values={configStore.config.partitions}
                placeholder="e.g. product, system_ext..."
                onChange={handlePartitionsChange}
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
                    <path d={ICONS.delete} />
                  </svg>
                </md-icon>
              </div>
              <div class="card-text">
                <span class="card-title">
                  {(uiStore.L.config as any).ignoreList ?? "Ignore List"}
                </span>
                <span class="card-desc">
                  {(uiStore.L.config as any).ignoreListDesc ??
                    "Directories to exclude from mounting"}
                </span>
              </div>
            </div>
            <div class="p-input">
              <ChipInput
                values={configStore.config.ignoreList}
                placeholder="/data/adb/modules/..."
                onChange={handleIgnoreListChange}
              />
            </div>
          </div>
        </section>

        <section class="config-group">
          <div class="config-card no-padding-v">
            <md-list>
              <md-list-item
                prop:type="button"
                on:click={() => toggle("umount")}
              >
                <div slot="start">
                  <md-icon>
                    <svg viewBox="0 0 24 24">
                      <path d={ICONS.anchor} />
                    </svg>
                  </md-icon>
                </div>
                <div slot="headline">{uiStore.L.config.umountLabel}</div>
                <div slot="supporting-text">
                  {configStore.config.umount
                    ? uiStore.L.config.umountOn
                    : uiStore.L.config.umountOff}
                </div>
                <md-switch
                  slot="end"
                  prop:selected={configStore.config.umount}
                  attr:touch-target="wrapper"
                />
              </md-list-item>
            </md-list>
          </div>
        </section>
      </div>

      <BottomActions>
        <md-filled-tonal-icon-button
          on:click={reload}
          prop:disabled={configStore.loading}
          prop:title={uiStore.L.config.reload}
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
          prop:disabled={configStore.saving || !isDirty()}
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
