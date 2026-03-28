/**
 * Copyright 2025 Magic Mount-rs Authors
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import { For, Show, createSignal } from "solid-js";

import "./ChipInput.css";

interface ChipInputProps {
  values: string[];
  placeholder?: string;
  onChange?: (values: string[]) => void;
}

export default function ChipInput(props: ChipInputProps) {
  const [inputValue, setInputValue] = createSignal("");

  function handleKeydown(e: KeyboardEvent) {
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
        props.onChange?.(newValues);
      }
      setInputValue("");
    }
  }

  function removeChip(index: number) {
    const newValues = props.values.filter((_, i) => i !== index);
    props.onChange?.(newValues);
  }

  return (
    <div class="chip-input-container">
      <For each={props.values}>
        {(val, i) => (
          <span class="chip">
            {val}
            <button
              class="chip-remove"
              onClick={() => removeChip(i())}
              tabindex="-1"
              aria-label={`Remove ${val}`}
            >
              <svg viewBox="0 0 24 24" width="14" height="14">
                <path
                  d="M19 6.41L17.59 5 12 10.59 6.41 5 5 6.41 10.59 12 5 17.59 6.41 19 12 13.41 17.59 19 19 17.59 13.41 12z"
                  fill="currentColor"
                />
              </svg>
            </button>
          </span>
        )}
      </For>
      <input
        type="text"
        class="chip-input-field"
        value={inputValue()}
        onInput={(e) => setInputValue(e.currentTarget.value)}
        onKeyDown={handleKeydown}
        onBlur={addChip}
        onFocus={(e) => {
          const target = e.currentTarget;
          setTimeout(() => {
            target.scrollIntoView({ behavior: "smooth", block: "center" });
          }, 300);
        }}
        placeholder={props.placeholder ?? "Add item..."}
        enterkeyhint="done"
      />
      <Show when={inputValue().trim().length > 0}>
        <button
          class="chip-add-btn"
          onClick={addChip}
          tabindex="-1"
          aria-label="Add chip"
        >
          <svg viewBox="0 0 24 24" width="20" height="20">
            <path
              d="M9 16.2L4.8 12l-1.4 1.4L9 19 21 7l-1.4-1.4L9 16.2z"
              fill="currentColor"
            />
          </svg>
        </button>
      </Show>
    </div>
  );
}
