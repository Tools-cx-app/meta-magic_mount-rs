/*
 * Copyright (C) 2026 meta-magic_mount-rs developers
 * SPDX-License-Identifier: GPL-v3
 */

import { ref } from "vue";
import { toast } from "kernelsu";
import { showSnackbar } from "miuix-vue";
import { getSupportedLocales, loadLocale, switchLocale } from "../../locales";

const lang = ref("en");
const isReady = ref(false);
const uiStyle = ref<"miuix" | "md3">("miuix");
const monetEnabled = ref(false);

const availableLanguages = ref<{ code: string; display: string }[]>([]);

async function fetchAvailableLanguages() {
  availableLanguages.value = await getSupportedLocales();
}

function showToast(text: string): void {
  if (uiStyle.value === "miuix") {
    showSnackbar({ message: text });
  } else {
    toast(text);
  }
}

async function setLang(code: string) {
  lang.value = code;
  await switchLocale(code);
}

function setUiStyle(style: "miuix" | "md3") {
  uiStyle.value = style;
  localStorage.setItem("uiStyle", style);
}

function setMonetEnabled(enabled: boolean) {
  monetEnabled.value = enabled;
  localStorage.setItem("monetEnabled", enabled ? "1" : "0");
  if (enabled) {
    document.documentElement.classList.add("miuix-monet");
  } else {
    document.documentElement.classList.remove("miuix-monet");
  }
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
  const savedMonet = localStorage.getItem("monetEnabled");
  if (savedMonet === "1") {
    monetEnabled.value = true;
    document.documentElement.classList.add("miuix-monet");
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
  get monetEnabled() {
    return monetEnabled.value;
  },
  showToast,
  setLang,
  setUiStyle,
  setMonetEnabled,
  init,
  fetchAvailableLanguages,
};
