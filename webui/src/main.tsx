/**
 * Copyright 2025 Magic Mount-rs Authors
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

/* @refresh reload */
import { render } from "solid-js/web";

import App from "./App";

import "./init";
import "./app.css";
import "./layout.css";

const root = document.querySelector("#app");

if (!root) {
  throw new Error("Root element not found");
}

render(() => <App />, root);
