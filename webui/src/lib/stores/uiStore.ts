/*
 * Copyright (C) 2026 Tools-cx-app <localhost.hutao@gmail.com>
 * SPDX-License-Identifier: Apache-2.0
 */

import { ref } from "vue";
import { showSnackbar } from "miuix-vue";
import { getSupportedLocales, loadLocale, switchLocale } from "../../locales";

const lang = ref("en");
const isReady = ref(false);

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

async function init() {
  const savedLang = localStorage.getItem("locale") ?? "en";
  await loadLocale(savedLang);
  lang.value = savedLang;
  localStorage.removeItem("mm-fix-nav");
  await fetchAvailableLanguages();
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
  showToast,
  setLang,
  init,
};
