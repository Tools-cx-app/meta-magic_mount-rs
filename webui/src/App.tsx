/**
 * Copyright 2025 Magic Mount-rs Authors
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import {
  For,
  Show,
  createEffect,
  createMemo,
  createSignal,
  lazy,
  onCleanup,
  onMount,
} from "solid-js";

import NavBar from "./components/NavBar";
import Spinner from "./components/Spinner";
import Toast from "./components/Toast";
import TopBar from "./components/TopBar";
import { configStore } from "./lib/stores/configStore";
import { moduleStore } from "./lib/stores/moduleStore";
import { sysStore } from "./lib/stores/sysStore";
import { uiStore } from "./lib/stores/uiStore";
import type { TabId } from "./lib/tabs";
import { TABS } from "./lib/tabs";

const loadStatusTab = () => import("./routes/StatusTab");
const loadConfigTab = () => import("./routes/ConfigTab");
const loadModulesTab = () => import("./routes/ModulesTab");
const loadInfoTab = () => import("./routes/InfoTab");

const routes = [
  { id: "status", load: loadStatusTab, component: lazy(loadStatusTab) },
  { id: "config", load: loadConfigTab, component: lazy(loadConfigTab) },
  { id: "modules", load: loadModulesTab, component: lazy(loadModulesTab) },
  { id: "info", load: loadInfoTab, component: lazy(loadInfoTab) },
] satisfies { id: TabId; load: () => Promise<unknown>; component: any }[];

export default function App() {
  const [activeTab, setActiveTab] = createSignal<TabId>("status");
  const [dragOffset, setDragOffset] = createSignal(0);
  const [isDragging, setIsDragging] = createSignal(false);
  const [containerWidth, setContainerWidth] = createSignal(0);
  const [visitedTabs, setVisitedTabs] = createSignal(
    new Set<TabId>(["status"]),
  );
  const [dragAxisLocked, setDragAxisLocked] = createSignal<"x" | "y" | null>(
    null,
  );
  let touchStartX = 0;
  let touchStartY = 0;

  function isEditingTarget(event: TouchEvent) {
    const activeElement = document.activeElement;
    if (
      activeElement instanceof HTMLElement &&
      (activeElement.matches("input, textarea, select") ||
        activeElement.isContentEditable ||
        !!activeElement.closest("md-outlined-text-field"))
    ) {
      return true;
    }

    for (const node of event.composedPath()) {
      if (!(node instanceof HTMLElement)) {
        continue;
      }
      if (
        node.matches("input, textarea, select, md-outlined-text-field") ||
        node.matches("[data-disable-tab-swipe='true']") ||
        !!node.closest("[data-disable-tab-swipe='true']") ||
        node.isContentEditable ||
        !!node.closest(
          "input, textarea, select, [contenteditable='true'], md-outlined-text-field",
        )
      ) {
        return true;
      }
    }

    return false;
  }

  function switchTab(id: TabId) {
    setActiveTab(id);
  }

  createEffect(() => {
    const currentTab = activeTab();
    setVisitedTabs((prev) => {
      if (prev.has(currentTab)) {
        return prev;
      }

      const next = new Set(prev);
      next.add(currentTab);

      return next;
    });
  });

  function handleTouchStart(e: TouchEvent) {
    if (isEditingTarget(e)) {
      setIsDragging(false);
      setDragOffset(0);

      return;
    }

    touchStartX = e.changedTouches[0].screenX;
    touchStartY = e.changedTouches[0].screenY;
    setIsDragging(true);
    setDragAxisLocked(null);
    setDragOffset(0);
  }

  function handleTouchMove(e: TouchEvent) {
    if (!isDragging()) {
      return;
    }
    const currentX = e.changedTouches[0].screenX;
    const currentY = e.changedTouches[0].screenY;
    let diffX = currentX - touchStartX;
    const diffY = currentY - touchStartY;

    if (dragAxisLocked() === null) {
      if (Math.abs(diffX) < 8 && Math.abs(diffY) < 8) {
        return;
      }
      setDragAxisLocked(Math.abs(diffX) >= Math.abs(diffY) ? "x" : "y");
    }

    if (dragAxisLocked() === "y") {
      setIsDragging(false);
      setDragOffset(0);

      return;
    }
    if (e.cancelable) {
      e.preventDefault();
    }
    const currentIndex = TABS.findIndex((t) => t.id === activeTab());
    if (
      (currentIndex === 0 && diffX > 0) ||
      (currentIndex === TABS.length - 1 && diffX < 0)
    ) {
      diffX = diffX / 3;
    }
    setDragOffset(diffX);
  }

  function handleTouchEnd() {
    if (!isDragging()) {
      setDragAxisLocked(null);

      return;
    }
    setIsDragging(false);
    const threshold = containerWidth() * 0.33 || 80;
    const currentIndex = TABS.findIndex((t) => t.id === activeTab());
    let nextIndex = currentIndex;
    if (dragOffset() < -threshold && currentIndex < TABS.length - 1) {
      nextIndex = currentIndex + 1;
    } else if (dragOffset() > threshold && currentIndex > 0) {
      nextIndex = currentIndex - 1;
    }
    if (nextIndex !== currentIndex) {
      switchTab(TABS[nextIndex].id);
    }
    setDragAxisLocked(null);
    setDragOffset(0);
  }

  onMount(async () => {
    let preloadTimer = 0;

    await uiStore.init();
    await Promise.all([
      configStore.loadConfig(),
      sysStore.ensureStatusLoaded(),
      moduleStore.ensureModulesLoaded(),
    ]);

    const pendingRoutes = routes.filter((route) => route.id !== activeTab());
    let nextIndex = 0;

    function preloadNextRoute() {
      const nextRoute = pendingRoutes[nextIndex++];
      if (!nextRoute) {
        return;
      }

      void nextRoute.load();

      if (nextIndex < pendingRoutes.length) {
        preloadTimer = window.setTimeout(preloadNextRoute, 120);
      }
    }

    preloadTimer = window.setTimeout(preloadNextRoute, 250);

    onCleanup(() => window.clearTimeout(preloadTimer));
  });

  const visibleTabs = createMemo(() => routes.map((route) => route.id));
  const baseTranslateX = createMemo(
    () =>
      visibleTabs().indexOf(activeTab()) *
      -(100 / Math.max(visibleTabs().length, 1)),
  );

  return (
    <div class="app-root">
      <Show
        when={uiStore.isReady}
        fallback={
          <div
            style={{
              "display": "flex",
              "justify-content": "center",
              "align-items": "center",
              "height": "100vh",
              "flex-direction": "column",
              "gap": "16px",
            }}
          >
            <Spinner />
            <span style={{ opacity: 0.6 }}>Loading...</span>
          </div>
        }
      >
        <TopBar />
        <main
          class="main-content"
          ref={(el) => {
            const observer = new ResizeObserver((entries) => {
              for (const entry of entries) {
                setContainerWidth(entry.contentRect.width);
              }
            });
            observer.observe(el);
          }}
          onTouchStart={handleTouchStart}
          onTouchMove={handleTouchMove}
          onTouchEnd={handleTouchEnd}
          onTouchCancel={handleTouchEnd}
        >
          <div
            class="swipe-track"
            style={{
              transform: `translateX(calc(${baseTranslateX()}% + ${dragOffset()}px))`,
              width: `${visibleTabs().length * 100}%`,
              transition: isDragging()
                ? "none"
                : "transform 0.3s cubic-bezier(0.25, 0.8, 0.5, 1)",
            }}
          >
            <For each={routes}>
              {(route) => (
                <div
                  class="swipe-page"
                  style={{
                    width: `${100 / Math.max(visibleTabs().length, 1)}%`,
                  }}
                >
                  <Show
                    when={visitedTabs().has(route.id)}
                    fallback={<div class="page-scroller" aria-hidden="true" />}
                  >
                    <div class="page-scroller">
                      <route.component />
                    </div>
                  </Show>
                </div>
              )}
            </For>
          </div>
        </main>
        <NavBar activeTab={activeTab()} onTabChange={switchTab} />
      </Show>
      <Toast />
    </div>
  );
}
