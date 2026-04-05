/**
 * Copyright 2025 Meta-Hybrid Mount Authors SPDX-License-Identifier:
 * GPL-3.0-or-later
 */

import { MdDialog } from "@material/web/dialog/dialog.js";
import type { DialogAnimation } from "@material/web/dialog/internal/animations.js";

const mdDialogPrototypePatched = Symbol("md-dialog-prototype-patched");
const mdDialogInstancePatched = Symbol("md-dialog-instance-patched");

type MdDialogPrototype = typeof MdDialog.prototype & {
  [mdDialogPrototypePatched]?: boolean;
};

type MdDialogElement = InstanceType<typeof MdDialog> & {
  [mdDialogInstancePatched]?: boolean;
};

const dialogOpenAnimation: DialogAnimation["dialog"] = [
  [
    [
      { opacity: 0, transform: "translateY(50px)" },
      { opacity: 1, transform: "translateY(0)" },
    ],
    { duration: 300, easing: "ease" },
  ],
];

const dialogCloseAnimation: DialogAnimation["dialog"] = [
  [
    [
      { opacity: 1, transform: "translateY(0)" },
      { opacity: 0, transform: "translateY(-50px)" },
    ],
    { duration: 300, easing: "ease" },
  ],
];

const scrimOpenAnimation: DialogAnimation["scrim"] = [
  [[{ opacity: 0 }, { opacity: 0.32 }], { duration: 300, easing: "linear" }],
];

const scrimCloseAnimation: DialogAnimation["scrim"] = [
  [[{ opacity: 0.32 }, { opacity: 0 }], { duration: 300, easing: "linear" }],
];

function applyMdDialogAnimationOverrides() {
  const prototype = MdDialog.prototype as MdDialogPrototype;

  if (prototype[mdDialogPrototypePatched]) {
    return;
  }

  function patchDialogInstance(dialog: MdDialogElement) {
    if (dialog[mdDialogInstancePatched]) {
      return;
    }

    const defaultOpenAnimation = dialog.getOpenAnimation.bind(dialog);
    const defaultCloseAnimation = dialog.getCloseAnimation.bind(dialog);

    dialog.getOpenAnimation = () => {
      const defaultAnimation = defaultOpenAnimation();

      return {
        ...defaultAnimation,
        dialog: dialogOpenAnimation,
        scrim: scrimOpenAnimation,
        container: [],
      };
    };

    dialog.getCloseAnimation = () => {
      const defaultAnimation = defaultCloseAnimation();

      return {
        ...defaultAnimation,
        dialog: dialogCloseAnimation,
        scrim: scrimCloseAnimation,
        container: [],
      };
    };

    dialog[mdDialogInstancePatched] = true;
  }

  document.querySelectorAll("md-dialog").forEach((dialog) => {
    if (dialog instanceof MdDialog) {
      patchDialogInstance(dialog);
    }
  });

  const observer = new MutationObserver((records) => {
    for (const record of records) {
      for (const node of record.addedNodes) {
        if (!(node instanceof HTMLElement)) {
          continue;
        }

        if (node instanceof MdDialog) {
          patchDialogInstance(node);
        }

        node.querySelectorAll("md-dialog").forEach((dialog) => {
          if (dialog instanceof MdDialog) {
            patchDialogInstance(dialog);
          }
        });
      }
    }
  });

  observer.observe(document.body, { childList: true, subtree: true });

  prototype[mdDialogPrototypePatched] = true;
}

if (customElements.get("md-dialog")) {
  applyMdDialogAnimationOverrides();
} else {
  void customElements.whenDefined("md-dialog").then(() => {
    applyMdDialogAnimationOverrides();
  });
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
