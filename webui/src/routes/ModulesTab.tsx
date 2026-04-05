/**
 * Copyright 2025 Magic Mount-rs Authors SPDX-License-Identifier:
 * GPL-3.0-or-later
 */

import { For, Show, createMemo, createSignal } from "solid-js";

import BottomActions from "../components/BottomActions";
import Skeleton from "../components/Skeleton";
import { ICONS } from "../lib/constants";
import { moduleStore } from "../lib/stores/moduleStore";
import { uiStore } from "../lib/stores/uiStore";

import "@material/web/button/filled-button.js";
import "@material/web/icon/icon.js";
import "@material/web/iconbutton/filled-tonal-icon-button.js";
import "./ModulesTab.css";

export default function ModulesTab() {
  const [searchQuery, setSearchQuery] = createSignal("");
  const [expandedId, setExpandedId] = createSignal<string | null>(null);

  const filteredModules = createMemo(() =>
    moduleStore.modules.filter((module) => {
      const query = searchQuery().toLowerCase();

      return (
        module.name.toLowerCase().includes(query) ||
        module.id.toLowerCase().includes(query)
      );
    }),
  );

  function toggleExpand(id: string) {
    setExpandedId(expandedId() === id ? null : id);
  }

  const getModeLabel = (isMounted: boolean) =>
    isMounted ? "Mounted" : "Unmounted";

  const getModeClass = (isMounted: boolean) =>
    isMounted ? "mode-mounted" : "mode-unmounted";

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
              placeholder={uiStore.L.modules.searchPlaceholder}
              value={searchQuery()}
              onInput={(event) => setSearchQuery(event.currentTarget.value)}
            />
          </div>
        </div>

        <div class="modules-list">
          <Show
            when={!moduleStore.loading}
            fallback={
              <For each={Array.from({ length: 6 })}>
                {() => <Skeleton class="skeleton-module-card" />}
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
                    {moduleStore.modules.length === 0
                      ? uiStore.L.modules.empty
                      : "No matching modules"}
                  </p>
                </div>
              }
            >
              <For each={filteredModules()}>
                {(module) => (
                  <div
                    class={`module-card ${expandedId() === module.id ? "expanded" : ""} ${module.is_mounted ? "" : "unmounted"}`}
                  >
                    <md-ripple />
                    <div
                      class="module-header"
                      onClick={() => toggleExpand(module.id)}
                      role="button"
                      tabIndex={0}
                      onKeyDown={(event) =>
                        (event.key === "Enter" || event.key === " ") &&
                        toggleExpand(module.id)
                      }
                    >
                      <div class="module-info">
                        <div class="module-name">{module.name}</div>
                        <div class="module-meta">
                          <span class="module-id">{module.id}</span>
                          <span class="version-badge">{module.version}</span>
                        </div>
                      </div>
                      <div
                        class={`mode-indicator ${getModeClass(module.is_mounted)}`}
                      >
                        {getModeLabel(module.is_mounted)}
                      </div>
                    </div>

                    <div class="module-body-wrapper">
                      <div class="module-body-inner">
                        <div class="module-body-content">
                          <div class="body-section">
                            <div class="section-label">
                              {uiStore.L.modules.descriptionLabel}
                            </div>
                            <p class="module-desc">
                              {module.description ??
                                uiStore.L.modules.noDescriptionLabel}
                            </p>
                          </div>

                          <div class="body-section">
                            <div class="section-label">
                              {uiStore.L.modules.authorLabel}
                            </div>
                            <div class="module-author">
                              {module.author ?? uiStore.L.modules.unknownLabel}
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
          onClick={() => {
            moduleStore.loadModules();
          }}
          title={uiStore.L.modules.reload}
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
