/**
 * Copyright 2026 Hybrid Mount Developers SPDX-License-Identifier:
 * GPL-3.0-or-later
 */

import "./Skeleton.css";

interface Props {
  class?: string;
}

export default (props: Props) => (
  <div class={`skeleton ${props.class ?? ""}`} />
);
