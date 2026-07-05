/*
 * Copyright (C) 2026 meta-magic_mount-rs developers
 * SPDX-License-Identifier: GPL-v3
 */

import { createApp } from "vue";
// import Vconsole from 'vconsole'
import i18n, { initI18n } from "./locales";
import "./style.css";

import App from "./App.vue";
import { uiStore } from "./lib/stores/uiStore";

const app = createApp(App);
app.use(i18n);
// new Vconsole()
const init = async () => {
  await uiStore.init();
  const savedLocale = localStorage.getItem("locale");
  await initI18n(savedLocale ?? undefined);
  app.mount("#app");
};

init();
