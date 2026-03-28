/**
 * Copyright 2025 Magic Mount-rs Authors
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import type { JSX } from "solid-js";
import { createSignal, onCleanup, onMount } from "solid-js";

import "./BottomActions.css";

interface BottomActionsProps {
  children: JSX.Element;
}

function isEditableElement(el: Element | null): boolean {
  if (!(el instanceof HTMLElement)) {
    return false;
  }

  if (el.isContentEditable) {
    return true;
  }

  return !!el.closest(
    "input, textarea, select, [contenteditable='true'], md-outlined-text-field",
  );
}

export default (props: BottomActionsProps) => {
  const [keyboardOpen, setKeyboardOpen] = createSignal(false);

  onMount(() => {
    const visualViewport = window.visualViewport;

    function updateKeyboardState() {
      const activeElement = document.activeElement;
      const isEditing = isEditableElement(activeElement);

      if (!visualViewport) {
        setKeyboardOpen(isEditing);

        return;
      }

      const keyboardHeight = window.innerHeight - visualViewport.height;
      setKeyboardOpen(isEditing && keyboardHeight > 120);
    }

    window.addEventListener("focusin", updateKeyboardState, true);
    window.addEventListener("focusout", updateKeyboardState, true);
    visualViewport?.addEventListener("resize", updateKeyboardState);
    visualViewport?.addEventListener("scroll", updateKeyboardState);
    updateKeyboardState();

    onCleanup(() => {
      window.removeEventListener("focusin", updateKeyboardState, true);
      window.removeEventListener("focusout", updateKeyboardState, true);
      visualViewport?.removeEventListener("resize", updateKeyboardState);
      visualViewport?.removeEventListener("scroll", updateKeyboardState);
    });
  });

  return (
    <div
      classList={{
        "bottom-actions-root": true,
        "is-keyboard-open": keyboardOpen(),
      }}
    >
      {props.children}
    </div>
  );
};
