/*
 * Copyright (C) 2026 Tools-cx-app <localhost.hutao@gmail.com>
 * SPDX-License-Identifier: Apache-2.0
 */

import { ref, computed } from "vue";
import type { ToastMessage } from "../types";
import { getSupportedLocales, loadLocale, switchLocale } from "../../locales";

const lang = ref("en");
const toast = ref<ToastMessage>({
  id: "init",
  text: "",
  type: "info",
  visible: false,
});
const isReady = ref(false);

const availableLanguages = ref<{ code: string; display: string }[]>([]);

async function fetchAvailableLanguages() {
  availableLanguages.value = await getSupportedLocales();
}

function showToast(text: string, type: ToastMessage["type"] = "info"): void {
  const id = Date.now().toString();
  toast.value = { id, text, type, visible: true };
  setTimeout(() => {
    if (toast.value.id === id) {
      toast.value.visible = false;
    }
  }, 3000);
}

async function setLang(code: string) {
  lang.value = code;
  await switchLocale(code);
}

async function init() {
  const savedLang = localStorage.getItem("locale") ?? "en";
  await loadLocale(savedLang);
  lang.value = savedLang;
  localStorage.removeItem("mm-fix-nav");
  await fetchAvailableLanguages();
  isReady.value = true;
}

const toasts = computed(() => {
  const t = toast.value;
  return t.visible ? [t] : [];
});

export const uiStore = {
  get lang() {
    return lang.value;
  },
  get availableLanguages() {
    return availableLanguages.value;
  },
  get toasts() {
    return toasts.value;
  },
  get isReady() {
    return isReady.value;
  },
  showToast,
  setLang,
  init,
};
