/**
 * Copyright 2025 Magic Mount-rs Authors
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import { For, Show, createSignal, onCleanup, onMount } from "solid-js";

import { ICONS } from "../lib/constants";
import { uiStore } from "../lib/stores/uiStore";

import "./TopBar.css";

export default function TopBar() {
  const [showLangMenu, setShowLangMenu] = createSignal(false);
  let langButtonRef: HTMLButtonElement | undefined;
  let menuRef: HTMLDivElement | undefined;

  function setLang(code: string) {
    uiStore.setLang(code);
    setShowLangMenu(false);
  }

  function handleOutsideClick(e: MouseEvent) {
    const target = e.target as Node;
    if (
      showLangMenu() &&
      menuRef &&
      !menuRef.contains(target) &&
      langButtonRef &&
      !langButtonRef.contains(target)
    ) {
      setShowLangMenu(false);
    }
  }

  onMount(() => {
    window.addEventListener("click", handleOutsideClick);
    onCleanup(() => window.removeEventListener("click", handleOutsideClick));
  });

  return (
    <header class="top-bar">
      <div class="top-bar-content">
        <h1 class="screen-title">{uiStore.L.common.appName}</h1>
        <div class="top-actions">
          <button
            class="btn-icon"
            ref={langButtonRef}
            onClick={() => setShowLangMenu(!showLangMenu())}
            title={uiStore.L.common.language}
          >
            <svg viewBox="0 0 24 24">
              <path d={ICONS.translate} fill="currentColor" />
            </svg>
          </button>

          <Show when={showLangMenu()}>
            <div class="menu-dropdown" ref={menuRef}>
              <For each={uiStore.availableLanguages}>
                {(l) => (
                  <button class="menu-item" onClick={() => setLang(l.code)}>
                    {l.name}
                  </button>
                )}
              </For>
            </div>
          </Show>
        </div>
      </div>
    </header>
  );
}
