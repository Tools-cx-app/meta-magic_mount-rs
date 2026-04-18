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

import { For, Show, createSignal } from "solid-js";

import "@material/web/chips/chip-set.js";
import "@material/web/chips/input-chip.js";
import "@material/web/icon/icon.js";
import "@material/web/iconbutton/icon-button.js";
import "./ChipInput.css";

interface Props {
  values: string[];
  placeholder?: string;
  onValuesChange: (newValues: string[]) => void;
}

export default function ChipInput(props: Props) {
  const [inputValue, setInputValue] = createSignal("");
  let inputRef: any = null;

  function handleKeyDown(e: KeyboardEvent) {
    if (e.key === "Enter" || e.key === "," || e.key === " ") {
      e.preventDefault();
      addChip();
    } else if (
      e.key === "Backspace" &&
      inputValue() === "" &&
      props.values.length > 0
    ) {
      removeChip(props.values.length - 1);
    }
  }

  function addChip() {
    const val = inputValue().trim();
    if (val) {
      if (!props.values.includes(val)) {
        const newValues = [...props.values, val];
        props.onValuesChange(newValues);
      }
      setInputValue("");
    }
  }

  function removeChip(index: number) {
    const newValues = props.values.filter((_, i) => i !== index);
    props.onValuesChange(newValues);
  }

  return (
    <div class="chip-input-wrapper">
      <md-chip-set
        classList={{ "chip-set-has-values": props.values.length > 0 }}
      >
        <For each={props.values}>
          {(val, i) => (
            <md-input-chip
              label={val}
              remove-only
              on:remove={() => removeChip(i())}
            />
          )}
        </For>
      </md-chip-set>

      <div class="input-row">
        <input
          ref={(el) => (inputRef = el)}
          type="text"
          class="chip-input-field"
          value={inputValue()}
          onInput={(e) => setInputValue(e.currentTarget.value)}
          onKeyDown={handleKeyDown}
          onBlur={addChip}
          onFocus={() => {
            setTimeout(() => {
              inputRef?.scrollIntoView({ behavior: "smooth", block: "center" });
            }, 300);
          }}
          placeholder={props.placeholder ?? "Add item..."}
          enterkeyhint="done"
        />
        <Show when={inputValue().trim().length > 0}>
          <md-icon-button onClick={addChip} class="add-btn" title="Add tag">
            <md-icon>
              <svg viewBox="0 0 24 24">
                <path d="M9 16.2L4.8 12l-1.4 1.4L9 19 21 7l-1.4-1.4L9 16.2z" />
              </svg>
            </md-icon>
          </md-icon-button>
        </Show>
      </div>
    </div>
  );
}
