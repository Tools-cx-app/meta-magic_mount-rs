/*
 * Copyright (C) 2026 Tools-cx-app <localhost.hutao@gmail.com>
 * SPDX-License-Identifier: Apache-2.0
 */

import "./Skeleton.css";

interface Props {
  class?: string;
}

export default (props: Props) => (
  <div class={`skeleton ${props.class ?? ""}`} />
);
