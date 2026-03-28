import { For, Show, createSignal } from "solid-js";

import "@material/web/chips/chip-set.js";
import "@material/web/chips/input-chip.js";
import "@material/web/icon/icon.js";
import "@material/web/iconbutton/icon-button.js";
import "./ChipInput.css";

interface ChipInputProps {
  values: string[];
  placeholder?: string;
  onChange?: (values: string[]) => void;
}

export default (props: ChipInputProps) => {
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

  function stopSwipePropagation(e: TouchEvent) {
    e.stopPropagation();
  }

  return (
    <div
      class="chip-input-wrapper"
      data-disable-tab-swipe="true"
      onTouchStart={stopSwipePropagation}
      onTouchMove={stopSwipePropagation}
      onTouchEnd={stopSwipePropagation}
      onTouchCancel={stopSwipePropagation}
    >
      <md-chip-set
        style={{ "margin-bottom": props.values.length > 0 ? "8px" : "0px" }}
      >
        <For each={props.values}>
          {(val, i) => (
            <md-input-chip
              prop:label={val}
              remove-only
              on:remove={() => removeChip(i())}
            />
          )}
        </For>
      </md-chip-set>

      <div class="input-row">
        <input
          type="text"
          class="chip-input-field"
          value={inputValue()}
          onInput={(e) => setInputValue(e.currentTarget.value)}
          onKeyDown={handleKeydown}
          onBlur={addChip}
          placeholder={props.placeholder ?? "Add item..."}
          enterkeyhint="done"
        />
        <Show when={inputValue().trim().length > 0}>
          <md-icon-button
            onClick={addChip}
            class="add-btn"
            role="button"
            tabIndex={0}
            title="Add tag"
          >
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
};
