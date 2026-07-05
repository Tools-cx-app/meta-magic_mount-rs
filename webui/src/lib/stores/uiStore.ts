/*
 * Copyright (C) 2026 meta-magic_mount-rs developers
 * SPDX-License-Identifier: GPL-v3
 */

import { ref } from "vue";
import { showSnackbar } from "miuix-vue";
import { getSupportedLocales, loadLocale, switchLocale } from "../../locales";

const lang = ref("en");
const isReady = ref(false);
const uiStyle = ref<"miuix" | "md3">("miuix");

const availableLanguages = ref<{ code: string; display: string }[]>([]);

async function fetchAvailableLanguages() {
  availableLanguages.value = await getSupportedLocales();
}

function showToast(text: string): void {
  showSnackbar({ message: text });
}

async function setLang(code: string) {
  lang.value = code;
  await switchLocale(code);
}

function setUiStyle(style: "miuix" | "md3") {
  uiStyle.value = style;
  localStorage.setItem("uiStyle", style);
}

async function init() {
  const savedLang = localStorage.getItem("locale") ?? "en";
  await loadLocale(savedLang);
  lang.value = savedLang;
  localStorage.removeItem("mm-fix-nav");
  await fetchAvailableLanguages();
  const savedStyle = localStorage.getItem("uiStyle") as
    "miuix" | "md3" | "custom" | null;
  if (savedStyle === "miuix" || savedStyle === "md3") {
    uiStyle.value = savedStyle;
  } else if (savedStyle === "custom") {
    uiStyle.value = "md3";
    localStorage.setItem("uiStyle", "md3");
  }
  console.log(uiStyle.value);
  isReady.value = true;
}

export const uiStore = {
  get lang() {
    return lang.value;
  },
  get availableLanguages() {
    return availableLanguages.value;
  },
  get isReady() {
    return isReady.value;
  },
  get uiStyle() {
    return uiStyle.value;
  },
  showToast,
  setLang,
  setUiStyle,
  init,
  fetchAvailableLanguages,
};
