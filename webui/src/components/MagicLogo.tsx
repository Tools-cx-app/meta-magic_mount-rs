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

import "./MagicLogo.css";

export default () => (
  <div class="logo-wrapper">
    <svg
      viewBox="0 0 160 210"
      xmlns="http://www.w3.org/2000/svg"
      class="magic-svg"
    >
      <g class="orbit-layer" transform="rotate(-25 80 82)">
        <path d="M 10,82 A 70,25 0 0,1 150,82" class="logo-orbit-track" />
        <g class="satellite-mover">
          <animateMotion
            dur="4s"
            begin="2.5s"
            repeatCount="indefinite"
            path="M 10,82 A 70,25 0 0,1 150,82 A 70,25 0 0,1 10,82"
          />
          <circle r="4.5" class="satellite-shape back-shape" />
        </g>
      </g>
      <g class="logo-network">
        <path d="M80 82 L56 34" class="logo-connector" />
        <path d="M80 82 L35 118" class="logo-connector" />
        <path d="M80 82 L134 92" class="logo-connector" />
        <path d="M80 82 L56 34" class="logo-flow logo-flow-top" />
        <path d="M80 82 L35 118" class="logo-flow logo-flow-left" />
        <path d="M80 82 L134 92" class="logo-flow logo-flow-right" />
        <circle cx="80" cy="82" r="26" class="logo-node logo-node-center" />
        <circle cx="56" cy="34" r="16" class="logo-node logo-node-top" />
        <circle cx="35" cy="118" r="16" class="logo-node logo-node-left" />
        <circle cx="134" cy="92" r="16" class="logo-node logo-node-right" />
        <path
          d="M75 120 H85 V172 H106 A16 16 0 0 1 122 188 H38 A16 16 0 0 1 54 172 H75 Z"
          class="logo-trunk"
        />
      </g>
      <g class="orbit-layer" transform="rotate(-25 80 82)">
        <path d="M 150,82 A 70,25 0 0,1 10,82" class="logo-orbit-track" />
        <g class="satellite-mover">
          <animateMotion
            dur="4s"
            begin="2.5s"
            repeatCount="indefinite"
            path="M 10,82 A 70,25 0 0,1 150,82 A 70,25 0 0,1 10,82"
          />
          <circle r="4.5" class="satellite-shape front-shape" />
        </g>
      </g>
    </svg>
  </div>
);
