import { For, createEffect } from "solid-js";

import { store } from "../lib/store";
import type { TabId } from "../lib/tabs";
import { TABS } from "../lib/tabs";

import "@material/web/icon/icon.js";
import "@material/web/ripple/ripple.js";
import "./NavBar.css";

interface NavBarProps {
  activeTab: TabId;
  onTabChange: (id: TabId) => void;
}

export default (props: NavBarProps) => {
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
    <nav
      class="bottom-nav"
      ref={navContainer}
      style={{
        "padding-bottom": store.fixBottomNav
          ? "48px"
          : "max(16px, env(safe-area-inset-bottom, 0px))",
      }}
    >
      <For each={TABS}>
        {(tab) => (
          <button
            class={`nav-tab ${props.activeTab === tab.id ? "active" : ""}`}
            onClick={() => props.onTabChange(tab.id)}
            ref={(el) => (tabRefs[tab.id] = el)}
            type="button"
          >
            <md-ripple />
            <div class="icon-container">
              <md-icon>
                <svg viewBox="0 0 24 24">
                  <path d={tab.icon} style={{ transition: "none" }} />
                </svg>
              </md-icon>
            </div>
            <span class="label">{store.L.tabs[tab.id]}</span>
          </button>
        )}
      </For>
    </nav>
  );
};
