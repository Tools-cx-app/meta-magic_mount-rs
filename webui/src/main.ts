/*
 * Copyright (C) 2026 meta-magic_mount-rs developers
 * SPDX-License-Identifier: Apache-2.0
 */

import { createApp } from "vue";
// import Vconsole from 'vconsole'
import i18n, { initI18n } from "./locales";
import "./style.css";
import "miuix-vue/style.css";
import App from "./App.vue";

const app = createApp(App);
app.use(i18n);
// new Vconsole() // unless need to debug,dont uncomment it
const init = async () => {
  const savedLocale = localStorage.getItem("locale");
  await initI18n(savedLocale ?? undefined);

  app.mount("#app");
};

init();
