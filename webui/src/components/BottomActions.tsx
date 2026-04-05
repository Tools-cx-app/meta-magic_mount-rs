import type { ParentProps } from "solid-js";
import { createEffect, createSignal, onCleanup, onMount } from "solid-js";
import { Portal } from "solid-js/web";

export default function BottomActions(props: ParentProps) {
  const [isActivePage, setIsActivePage] = createSignal(true);
  const [_, setKeyboardInset] = createSignal(0);
  let anchorRef: HTMLDivElement | undefined;
  let rootRef: HTMLDivElement | undefined;

  onMount(() => {
    const pageEl = anchorRef?.closest(".swipe-page");
    const rootEl = anchorRef?.closest(".main-content");
    if (!pageEl || !rootEl) {
      return;
    }

    const observer = new IntersectionObserver(
      ([entry]) => {
        setIsActivePage(entry.isIntersecting && entry.intersectionRatio >= 0.6);
      },
      {
        root: rootEl,
        threshold: [0.6],
      },
    );

    observer.observe(pageEl);
    onCleanup(() => observer.disconnect());
  });

  onMount(() => {
    const maybeViewport = window.visualViewport;
    if (!maybeViewport) {
      return;
    }

    const visualViewport = maybeViewport;

    let rafId = 0;
    function updateKeyboardInset() {
      if (rafId) {
        return;
      }

      rafId = window.requestAnimationFrame(() => {
        rafId = 0;
        const inset = Math.max(
          0,
          Math.round(
            window.innerHeight -
              visualViewport.height -
              visualViewport.offsetTop,
          ),
        );
        setKeyboardInset((prev) => (Math.abs(prev - inset) < 2 ? prev : inset));
      });
    }

    updateKeyboardInset();
    visualViewport.addEventListener("resize", updateKeyboardInset);
    visualViewport.addEventListener("scroll", updateKeyboardInset);
    window.addEventListener("orientationchange", updateKeyboardInset);

    onCleanup(() => {
      if (rafId) {
        window.cancelAnimationFrame(rafId);
      }

      visualViewport.removeEventListener("resize", updateKeyboardInset);
      visualViewport.removeEventListener("scroll", updateKeyboardInset);
      window.removeEventListener("orientationchange", updateKeyboardInset);
    });
  });

  createEffect(() => {
    if (!rootRef) {
      return;
    }
    rootRef.toggleAttribute("inert", !isActivePage());
  });

  return (
    <>
      <div class="bottom-actions-anchor" ref={anchorRef} aria-hidden="true" />
      <Portal>
        <div
          ref={rootRef}
          class="bottom-actions-root"
          classList={{ "is-active": isActivePage() }}
        >
          {props.children}
        </div>
      </Portal>
    </>
  );
}
