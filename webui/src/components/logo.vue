<!--

    Copyright (C) 2026 Tools-cx-app <localhost.hutao@gmail.com>
    SPDX-License-Identifier: Apache-2.0

-->
<template>
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
</template>

<style>
.logo-wrapper {
  --logo-accent: var(--m-color-primary);
  --logo-flow: var(--m-color-tertiary-container);
  --logo-satellite: var(--m-color-error);
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
}
.magic-svg {
  width: 100%;
  height: 100%;
  max-width: 160px;
  max-height: 210px;
  overflow: visible;
  shape-rendering: geometricPrecision;
}
.logo-connector,
.logo-trunk {
  fill: none;
  stroke: var(--logo-accent);
  stroke-linecap: round;
  stroke-linejoin: round;
}
.logo-connector {
  stroke-width: 14;
  opacity: 0.96;
  stroke-dasharray: 70;
  stroke-dashoffset: 70;
  animation: draw-connector 0.6s ease-out forwards;
}
.logo-connector:nth-of-type(1) {
  animation-delay: 0.3s;
}
.logo-connector:nth-of-type(2) {
  animation-delay: 0.4s;
}
.logo-connector:nth-of-type(3) {
  animation-delay: 0.5s;
}
.logo-trunk {
  fill: var(--logo-accent);
  stroke: none;
  transform-origin: 80px 188px;
  transform: scaleY(0);
  animation: grow-trunk 0.5s cubic-bezier(0.175, 0.885, 0.32, 1.275) forwards;
}
.logo-node,
.logo-network {
  fill: var(--logo-accent);
}
.logo-node {
  transform-origin: center;
  transform-box: fill-box;
  transform: scale(0);
  animation:
    pop-node 0.5s cubic-bezier(0.175, 0.885, 0.32, 1.275) forwards,
    breathe-node 3s ease-in-out infinite alternate;
  transition: filter 0.3s ease;
}
.logo-node-center {
  animation-delay: 0.1s, 1.2s;
}
.logo-node-top {
  animation-delay: 0.6s, 1.5s;
}
.logo-node-left {
  animation-delay: 0.7s, 1.8s;
}
.logo-node-right {
  animation-delay: 0.8s, 2.1s;
}
.logo-flow {
  fill: none;
  stroke: var(--logo-flow);
  stroke-width: 4;
  stroke-dasharray: 14 52;
  stroke-linecap: round;
  opacity: 0;
  animation:
    fade-in 0.3s ease forwards,
    flow 2s linear infinite;
  transition:
    stroke-width 0.3s ease,
    filter 0.3s ease;
}
.logo-flow-top {
  animation-delay: 1.2s, 1.2s;
}
.logo-flow-left {
  animation-delay: 1.2s, 0.2s;
}
.logo-flow-right {
  animation-delay:
    1.2s,
    -0.8s;
}
.orbit-layer {
  opacity: 0;
  animation: full-fade-in 0.8s 2.5s ease forwards;
}
.logo-orbit-track {
  fill: none;
  stroke: var(--logo-flow);
  stroke-width: 1.5;
  stroke-dasharray: 3 6;
  opacity: 0.3;
}
.satellite-shape {
  fill: var(--logo-satellite);
  transform-box: fill-box;
  transform-origin: center;
}
.back-shape {
  animation: satellite-back 4s linear 2.5s infinite backwards;
}
.front-shape {
  animation: satellite-front 4s linear 2.5s infinite backwards;
}
@keyframes draw-connector {
  to {
    stroke-dashoffset: 0;
  }
}
@keyframes grow-trunk {
  to {
    transform: scaleY(1);
  }
}
@keyframes pop-node {
  to {
    transform: scale(1);
  }
}
@keyframes breathe-node {
  0% {
    transform: scale(1);
  }
  100% {
    transform: scale(1.08);
  }
}
@keyframes fade-in {
  to {
    opacity: 0.72;
  }
}
@keyframes full-fade-in {
  to {
    opacity: 1;
  }
}
@keyframes flow {
  from {
    stroke-dashoffset: 66;
  }
  to {
    stroke-dashoffset: 0;
  }
}
@keyframes satellite-back {
  0% {
    transform: scale(1);
    opacity: 1;
  }
  25% {
    transform: scale(0.6);
    opacity: 0.3;
  }
  49.9% {
    transform: scale(1);
    opacity: 1;
  }
  50% {
    opacity: 0;
  }
  100% {
    opacity: 0;
  }
}
@keyframes satellite-front {
  0% {
    opacity: 0;
  }
  49.9% {
    opacity: 0;
  }
  50% {
    transform: scale(1);
    opacity: 1;
  }
  75% {
    transform: scale(1.4);
    opacity: 1;
    filter: drop-shadow(0 0 6px var(--logo-satellite));
  }
  100% {
    transform: scale(1);
    opacity: 1;
  }
}
</style>

export default { name: 'MagicLogo', };
