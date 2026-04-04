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
import Toast from "./components/Toast";
import TopBar from "./components/TopBar";
import { configStore } from "./lib/stores/configStore";
import { moduleStore } from "./lib/stores/moduleStore";
import { sysStore } from "./lib/stores/sysStore";
import { uiStore } from "./lib/stores/uiStore";

const loadStatusTab = () => import("./routes/StatusTab");
const loadConfigTab = () => import("./routes/ConfigTab");
const loadModulesTab = () => import("./routes/ModulesTab");
const loadInfoTab = () => import("./routes/InfoTab");
type TabId = "status" | "config" | "modules" | "info";

const routes = [
  { id: "status", load: loadStatusTab, component: lazy(loadStatusTab) },
  { id: "config", load: loadConfigTab, component: lazy(loadConfigTab) },
  { id: "modules", load: loadModulesTab, component: lazy(loadModulesTab) },
  { id: "info", load: loadInfoTab, component: lazy(loadInfoTab) },
] as const;

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

  const visibleTabs = createMemo(() => routes.map((r) => r.id));

  const baseTranslateX = createMemo(() => {
    const index = visibleTabs().indexOf(activeTab());

    return index * -(100 / visibleTabs().length);
  });

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

    const tabs = visibleTabs();
    const currentIndex = tabs.indexOf(activeTab());

    if (
      (currentIndex === 0 && diffX > 0) ||
      (currentIndex === tabs.length - 1 && diffX < 0)
    ) {
      diffX /= 3;
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
    const tabs = visibleTabs();
    const currentIndex = tabs.indexOf(activeTab());
    let nextIndex = currentIndex;

    if (dragOffset() < -threshold && currentIndex < tabs.length - 1) {
      nextIndex = currentIndex + 1;
    } else if (dragOffset() > threshold && currentIndex > 0) {
      nextIndex = currentIndex - 1;
    }

    if (nextIndex !== currentIndex) {
      setActiveTab(tabs[nextIndex]);
    }

    setDragAxisLocked(null);
    setDragOffset(0);
  }

  onMount(async () => {
    await uiStore.init();
    await Promise.all([
      configStore.loadConfig(),
      sysStore.ensureStatusLoaded(),
      moduleStore.ensureModulesLoaded(),
    ]);

    const pendingRoutes = routes.filter((route) => route.id !== activeTab());
    let preloadTimer = 0;
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

  return (
    <div class="app-root">
      <Show
        when={uiStore.isReady}
        fallback={
          <div class="loading-container">
            <div class="spinner" />
            <span class="loading-text">Loading...</span>
          </div>
        }
      >
        <TopBar />
        <main
          class="main-content"
          ref={(element) => {
            const observer = new ResizeObserver((entries) => {
              for (const entry of entries) {
                setContainerWidth(entry.contentRect.width);
              }
            });
            observer.observe(element);
            onCleanup(() => observer.disconnect());
          }}
          onTouchStart={handleTouchStart}
          onTouchMove={handleTouchMove}
          onTouchEnd={handleTouchEnd}
          onTouchCancel={handleTouchEnd}
        >
          <div
            class="swipe-track"
            style={{
              "--tab-count": visibleTabs().length,
              "--swipe-base": `${baseTranslateX()}%`,
              "--swipe-offset": `${dragOffset()}px`,
              "--swipe-transition": isDragging()
                ? "none"
                : "transform 0.4s cubic-bezier(0.2, 1, 0.2, 1)",
            }}
          >
            <For each={routes}>
              {(route) => (
                <div class="swipe-page">
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
        <NavBar
          activeTab={activeTab()}
          onTabChange={setActiveTab as (id: string) => void}
          tabs={routes}
        />
      </Show>
      <Toast />
    </div>
  );
}
