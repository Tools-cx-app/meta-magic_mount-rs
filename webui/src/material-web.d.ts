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

import type { JSX } from "solid-js";

type BaseProps = JSX.HTMLAttributes<HTMLElement>;

interface MdDialogProps extends BaseProps {
  open?: boolean;
  onclose?: (e: Event) => void;
}

interface MdTextFieldProps extends BaseProps {
  "label"?: string;
  "placeholder"?: string;
  "value"?: string;
  "error"?: boolean;
  "supporting-text"?: string;
  "disabled"?: boolean;
  "type"?: string;
  "onInput"?: (e: InputEvent) => void;
}

interface MdButtonProps extends BaseProps {
  disabled?: boolean;
  type?: string;
  href?: string;
  target?: string;
  onClick?: (e: MouseEvent) => void;
}

interface MdIconButtonProps extends BaseProps {
  disabled?: boolean;
  type?: string;
  href?: string;
  target?: string;
  onClick?: (e: MouseEvent) => void;
}

interface MdChipProps extends BaseProps {
  "label"?: string;
  "selected"?: boolean;
  "elevated"?: boolean;
  "remove-only"?: boolean;
  "on:remove"?: (e: Event) => void;
}

interface MdListItemProps extends BaseProps {
  type?: string;
  href?: string;
  target?: string;
  disabled?: boolean;
}

declare module "solid-js" {
  namespace JSX {
    interface IntrinsicElements {
      "md-icon": BaseProps;
      "md-icon-button": MdIconButtonProps;
      "md-filled-tonal-icon-button": MdIconButtonProps;
      "md-filled-button": MdButtonProps;
      "md-text-button": MdButtonProps;
      "md-filled-tonal-button": MdButtonProps;
      "md-outlined-text-field": MdTextFieldProps;
      "md-dialog": MdDialogProps;
      "md-linear-progress": BaseProps & {
        value?: number;
        indeterminate?: boolean;
      };
      "md-chip-set": BaseProps;
      "md-filter-chip": MdChipProps;
      "md-input-chip": MdChipProps;
      "md-ripple": BaseProps;
      "md-list": BaseProps;
      "md-list-item": MdListItemProps;
      "md-switch": BaseProps & { selected?: boolean };
      "md-divider": BaseProps;
    }
  }
}
