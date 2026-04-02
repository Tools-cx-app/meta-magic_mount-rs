/**
 * Copyright 2025 Magic Mount-rs Authors
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import { For, createEffect } from "solid-js";

import { uiStore } from "../lib/stores/uiStore";
import type { TabId } from "../lib/tabs";
import { TABS } from "../lib/tabs";

import "./NavBar.css";

interface NavBarProps {
  activeTab: TabId;
  onTabChange: (id: TabId) => void;
}

export default function NavBar(props: NavBarProps) {
  let navContainer: HTMLElement | undefined;
  const tabRefs: Record<string, HTMLElement | undefined> = {};

  createEffect(() => {
    const activeTab = props.activeTab;
    if (activeTab && tabRefs[activeTab] && navContainer) {
      const tab = tabRefs[activeTab];
      const containerWidth = navContainer.clientWidth;
      const tabLeft = tab.offsetLeft;
      const tabWidth = tab.clientWidth;
      const scrollLeft = tabLeft - containerWidth / 2 + tabWidth / 2;
      navContainer.scrollTo({
        left: scrollLeft,
        behavior: "smooth",
      });
    }
  });

  return (
    <nav class="bottom-nav" ref={navContainer}>
      <For each={TABS}>
        {(tab) => (
          <button
            class={`nav-tab ${props.activeTab === tab.id ? "active" : ""}`}
            onClick={() => props.onTabChange(tab.id)}
            ref={(el) => (tabRefs[tab.id] = el)}
            type="button"
          >
            <div class="icon-container">
              <svg viewBox="0 0 24 24">
                <path d={tab.icon} />
              </svg>
            </div>
            <span class="label">{uiStore.L.tabs[tab.id]}</span>
          </button>
        )}
      </For>
    </nav>
  );
}
