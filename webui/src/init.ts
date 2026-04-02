/**
 * Copyright 2025 Magic Mount-rs Authors
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

declare global {
  interface Window {
    litDisableBundleWarning: boolean;
  }
}

window.litDisableBundleWarning = true;

const viewportContent =
  "width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no, viewport-fit=cover";
let meta = document.querySelector('meta[name="viewport"]');

if (!meta) {
  meta = document.createElement("meta");
  meta.setAttribute("name", "viewport");
  document.head.appendChild(meta);
}

meta.setAttribute("content", viewportContent);

document.addEventListener(
  "touchmove",
  (event) => {
    if (event.touches.length > 1) {
      event.preventDefault();
    }
  },
  { passive: false },
);

document.addEventListener("gesturestart", (event) => {
  event.preventDefault();
});

export const materialWebInitApplied = true;
