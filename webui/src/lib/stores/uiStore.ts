/*
 * Copyright (C) 2026 Tools-cx-app <localhost.hutao@gmail.com>
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

import { createMemo, createRoot, createSignal } from "solid-js";

import type { LanguageOption, ToastMessage } from "../../types";
import { availableLanguages, locales } from "../i18n";

function createUiStore() {
  const [lang, setLangSignal] = createSignal("en");
  const [toast, setToast] = createSignal<ToastMessage>({
    id: "init",
    text: "",
    type: "info",
    visible: false,
  });
  const [isReady, setIsReady] = createSignal(false);

  const L = createMemo(() => locales[lang()] ?? locales.en);
  const toasts = createMemo(() => {
    const t = toast();

    return t.visible ? [t] : [];
  });

  function showToast(text: string, type: ToastMessage["type"] = "info"): void {
    const id = Date.now().toString();
    setToast({ id, text, type, visible: true });
    setTimeout(() => {
      if (toast().id === id) {
        setToast((prev) => ({ ...prev, visible: false }));
      }
    }, 3000);
  }

  function setLang(code: string) {
    setLangSignal(code);
    localStorage.setItem("mm-lang", code);
  }

  async function init() {
    const savedLang = localStorage.getItem("mm-lang") ?? "en";
    setLangSignal(savedLang);
    localStorage.removeItem("mm-fix-nav");
    setIsReady(true);
  }

  return {
    get lang() {
      return lang();
    },
    get availableLanguages(): LanguageOption[] {
      return availableLanguages;
    },
    get L() {
      return L();
    },
    get toasts() {
      return toasts();
    },
    get isReady() {
      return isReady();
    },
    showToast,
    setLang,
    init,
  };
}

export const uiStore = createRoot(createUiStore);
