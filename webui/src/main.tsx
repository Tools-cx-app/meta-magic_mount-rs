/*
 * Copyright (C) 2026 Tools-cx-app <localhost.hutao@gmail.com>
 * SPDX-License-Identifier: Apache-2.0
 */

import { render } from "solid-js/web";

import App from "./App.tsx";

import "./init";
import "./app.css";
import "./layout.css";

const root = document.querySelector("#app");

if (root instanceof HTMLElement) {
  render(() => <App />, root);
}
