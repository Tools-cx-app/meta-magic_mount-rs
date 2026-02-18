/**
 * Copyright 2025 Magic Mount-rs Authors
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import { Show, createMemo, createSignal, onMount } from "solid-js";

import NavBar from "./components/NavBar";
import Spinner from "./components/Spinner";
import Toast from "./components/Toast";
import TopBar from "./components/TopBar";
import { store } from "./lib/store";
import type { TabId } from "./lib/tabs";
import { TABS } from "./lib/tabs";
import ConfigTab from "./routes/ConfigTab";
import InfoTab from "./routes/InfoTab";
import ModulesTab from "./routes/ModulesTab";
import StatusTab from "./routes/StatusTab";

export default function App() {
  const [activeTab, setActiveTab] = createSignal<TabId>("status");
  const [dragOffset, setDragOffset] = createSignal(0);
  const [isDragging, setIsDragging] = createSignal(false);
  const [containerWidth, setContainerWidth] = createSignal(0);
  const [isReady, setIsReady] = createSignal(false);
  let touchStartX = 0;
  let touchStartY = 0;

  function switchTab(id: TabId) {
    setActiveTab(id);
  }

  function handleTouchStart(e: TouchEvent) {
    touchStartX = e.changedTouches[0].screenX;
    touchStartY = e.changedTouches[0].screenY;
    setIsDragging(true);
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
    if (Math.abs(diffY) > Math.abs(diffX)) {
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
    setDragOffset(0);
  }

  onMount(async () => {
    try {
      await store.init();
    } finally {
      setIsReady(true);
    }
  });

  const baseTranslateX = createMemo(
    () => TABS.findIndex((t) => t.id === activeTab()) * -20,
  );

  return (
    <div class="app-root">
      <Show
        when={isReady()}
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
              transition: isDragging()
                ? "none"
                : "transform 0.3s cubic-bezier(0.25, 0.8, 0.5, 1)",
            }}
          >
            <div class="swipe-page">
              <div class="page-scroller">
                <StatusTab />
              </div>
            </div>
            <div class="swipe-page">
              <div class="page-scroller">
                <ConfigTab />
              </div>
            </div>
            <div class="swipe-page">
              <div class="page-scroller">
                <ModulesTab />
              </div>
            </div>
            <div class="swipe-page">
              <div class="page-scroller">
                <InfoTab />
              </div>
            </div>
          </div>
        </main>
        <NavBar activeTab={activeTab()} onTabChange={switchTab} />
      </Show>
      <Toast />
    </div>
  );
}
