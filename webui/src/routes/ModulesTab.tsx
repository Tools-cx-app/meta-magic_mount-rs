/**
 * Copyright 2025 Magic Mount-rs Authors
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import { For, Show, createMemo, createSignal, onMount } from "solid-js";

import BottomActions from "../components/BottomActions";
import Skeleton from "../components/Skeleton";
import { ICONS } from "../lib/constants";
import { store } from "../lib/store";

import "./ModulesTab.css";
import "@material/web/iconbutton/filled-tonal-icon-button.js";
import "@material/web/button/filled-button.js";
import "@material/web/icon/icon.js";

export default function ModulesTab() {
  const [searchQuery, setSearchQuery] = createSignal("");
  const [expandedId, setExpandedId] = createSignal<string | null>(null);

  onMount(() => {
    store.loadModules();
  });

  const filteredModules = createMemo(() =>
    store.modules.filter((m) => {
      const q = searchQuery().toLowerCase();
      const matchSearch =
        m.name.toLowerCase().includes(q) || m.id.toLowerCase().includes(q);
      return matchSearch;
    }),
  );

  function toggleExpand(id: string) {
    setExpandedId(expandedId() === id ? null : id);
  }

  function getModeLabel(isMounted: boolean) {
    return isMounted ? "Mounted" : "Unmounted";
  }

  function getModeClass(isMounted: boolean) {
    return isMounted ? "mode-mounted" : "mode-unmounted";
  }

  return (
    <>
      <div class="modules-page">
        <div class="header-section">
          <div class="search-bar">
            <svg class="search-icon" viewBox="0 0 24 24">
              <path d={ICONS.search} />
            </svg>
            <input
              type="text"
              class="search-input"
              placeholder={store.L.modules.searchPlaceholder}
              value={searchQuery()}
              onInput={(e) => setSearchQuery(e.currentTarget.value)}
            />
          </div>
        </div>

        <div class="modules-list">
          <Show
            when={!store.loading.modules}
            fallback={
              <For each={Array(6)}>
                {() => <Skeleton height="64px" borderRadius="16px" />}
              </For>
            }
          >
            <Show
              when={filteredModules().length > 0}
              fallback={
                <div class="empty-state">
                  <div class="empty-icon">
                    <md-icon>
                      <svg viewBox="0 0 24 24">
                        <path d={ICONS.modules} />
                      </svg>
                    </md-icon>
                  </div>
                  <p>
                    {store.modules.length === 0
                      ? store.L.modules.empty
                      : "No matching modules"}
                  </p>
                </div>
              }
            >
              <For each={filteredModules()}>
                {(mod) => (
                  <div
                    class={`module-card ${expandedId() === mod.id ? "expanded" : ""} ${!mod.is_mounted ? "unmounted" : ""}`}
                  >
                    <div
                      class="module-header"
                      onClick={() => toggleExpand(mod.id)}
                      role="button"
                      tabIndex={0}
                      onKeyDown={(e) =>
                        (e.key === "Enter" || e.key === " ") &&
                        toggleExpand(mod.id)
                      }
                    >
                      <div class="module-info">
                        <div class="module-name">{mod.name}</div>
                        <div class="module-meta">
                          <span class="module-id">{mod.id}</span>
                          <span class="version-badge">{mod.version}</span>
                        </div>
                      </div>
                      <div class={`mode-indicator ${getModeClass(mod.is_mounted)}`}>
                        {getModeLabel(mod.is_mounted)}
                      </div>
                    </div>

                    <div class="module-body-wrapper">
                      <div class="module-body-inner">
                        <div class="module-body-content">
                          <div class="body-section">
                            <div class="section-label">
                              {store.L.modules.descriptionLabel}
                            </div>
                            <p class="module-desc">
                              {mod.description ||
                                store.L.modules.noDescriptionLabel}
                            </p>
                          </div>

                          <div class="body-section">
                            <div class="section-label">
                              {store.L.modules.authorLabel}
                            </div>
                            <div class="module-author">
                              {mod.author || store.L.modules.unknownLabel}
                            </div>
                          </div>
                        </div>
                      </div>
                    </div>
                  </div>
                )}
              </For>
            </Show>
          </Show>
        </div>
      </div>

      <BottomActions>
        <div class="spacer" />
        <md-filled-tonal-icon-button
          onClick={() => store.loadModules()}
          title={store.L.modules.reload}
        >
          <md-icon>
            <svg viewBox="0 0 24 24">
              <path d={ICONS.refresh} />
            </svg>
          </md-icon>
        </md-filled-tonal-icon-button>
      </BottomActions>
    </>
  );
}