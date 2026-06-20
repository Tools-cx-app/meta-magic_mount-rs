/*
 * Copyright (C) 2026 meta-magic_mount-rs developers
 * SPDX-License-Identifier: Apache-2.0
 */

import "./Skeleton.css";

interface Props {
  class?: string;
}

export default (props: Props) => (
  <div class={`skeleton ${props.class ?? ""}`} />
);
