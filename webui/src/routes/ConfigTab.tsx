import { createEffect, createMemo, createSignal } from "solid-js";

import BottomActions from "../components/BottomActions";
import ChipInput from "../components/ChipInput";
import { ICONS } from "../lib/constants";
import { configStore } from "../lib/stores/configStore";
import { uiStore } from "../lib/stores/uiStore";
import type { AppConfig } from "../types";

import "@material/web/button/filled-button.js";
import "@material/web/icon/icon.js";
import "@material/web/iconbutton/filled-tonal-icon-button.js";
import "@material/web/ripple/ripple.js";
import "@material/web/textfield/outlined-text-field.js";
import "./ConfigTab.css";

export default function ConfigTab() {
  const [initialConfigStr, setInitialConfigStr] = createSignal("");
  let mountSourceInputRef: any = null;

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
