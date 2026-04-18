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

import { Show, createMemo, createSignal } from "solid-js";

import BottomActions from "../components/BottomActions";
import Skeleton from "../components/Skeleton";
import { ICONS } from "../lib/constants";
import { configStore } from "../lib/stores/configStore";
import { moduleStore } from "../lib/stores/moduleStore";
import { sysStore } from "../lib/stores/sysStore";
import { uiStore } from "../lib/stores/uiStore";

import "@material/web/button/text-button.js";
import "@material/web/dialog/dialog.js";
import "@material/web/icon/icon.js";
import "@material/web/iconbutton/filled-tonal-icon-button.js";
import "./StatusTab.css";

export default function StatusTab() {
  const [showRebootConfirm, setShowRebootConfirm] = createSignal(false);

  const mountedCount = createMemo(
    () => moduleStore.modules.filter((module) => module.is_mounted).length,
  );

  const moduleStatsReady = createMemo(
    () => !moduleStore.loading && moduleStore.hasLoaded,
  );

  function reboot() {
    setShowRebootConfirm(false);
    void sysStore.rebootDevice();
  }

  return (
    <>
      <div class="dialog-container">
        <md-dialog
          open={showRebootConfirm()}
          onClose={() => setShowRebootConfirm(false)}
        >
          <div slot="headline">{uiStore.L.common.rebootTitle}</div>
          <div slot="content">{uiStore.L.common.rebootConfirm}</div>
          <div slot="actions">
            <md-text-button onClick={() => setShowRebootConfirm(false)}>
              {uiStore.L.common.cancel}
            </md-text-button>
            <md-text-button onClick={reboot}>
              {uiStore.L.common.reboot}
            </md-text-button>
          </div>
        </md-dialog>
      </div>

      <div class="dashboard-grid">
        <div class="hero-card">
          <div class="hero-bg-decoration">
            <svg class="hero-corner-blossom" viewBox="0 0 120 120">
              <defs>
                <path
                  id="hero-blossom-petal-shape"
                  d="M60 8C73 8 84 20 84 34C84 52 71 64 64 72C62 74 61 77 60 82C59 77 58 74 56 72C49 64 36 52 36 34C36 20 47 8 60 8Z"
                />
                <mask id="hero-blossom-outline-mask">
                  <rect width="120" height="120" fill="white" />
                  <g fill="black">
                    <use href="#hero-blossom-petal-shape" />
                    <use
                      href="#hero-blossom-petal-shape"
                      transform="rotate(72 60 60)"
                    />
                    <use
                      href="#hero-blossom-petal-shape"
                      transform="rotate(144 60 60)"
                    />
                    <use
                      href="#hero-blossom-petal-shape"
                      transform="rotate(216 60 60)"
                    />
                    <use
                      href="#hero-blossom-petal-shape"
                      transform="rotate(288 60 60)"
                    />
                  </g>
                </mask>
              </defs>
              <g class="hero-blossom-petals">
                <use href="#hero-blossom-petal-shape" />
                <use
                  href="#hero-blossom-petal-shape"
                  transform="rotate(72 60 60)"
                />
                <use
                  href="#hero-blossom-petal-shape"
                  transform="rotate(144 60 60)"
                />
                <use
                  href="#hero-blossom-petal-shape"
                  transform="rotate(216 60 60)"
                />
                <use
                  href="#hero-blossom-petal-shape"
                  transform="rotate(288 60 60)"
                />
              </g>
              <g class="hero-blossom-outline">
                <use href="#hero-blossom-petal-shape" />
                <use
                  href="#hero-blossom-petal-shape"
                  transform="rotate(72 60 60)"
                />
                <use
                  href="#hero-blossom-petal-shape"
                  transform="rotate(144 60 60)"
                />
                <use
                  href="#hero-blossom-petal-shape"
                  transform="rotate(216 60 60)"
                />
                <use
                  href="#hero-blossom-petal-shape"
                  transform="rotate(288 60 60)"
                />
              </g>
              <g class="hero-blossom-core">
                <circle cx="60" cy="60" r="2" />
              </g>
              <g class="hero-blossom-stamens">
                <path d="M60 52Q52 41 60 30" />
                <path d="M60 52Q52 41 60 30" transform="rotate(72 60 60)" />
                <path d="M60 52Q52 41 60 30" transform="rotate(144 60 60)" />
                <path d="M60 52Q52 41 60 30" transform="rotate(216 60 60)" />
                <path d="M60 52Q52 41 60 30" transform="rotate(288 60 60)" />
                <circle cx="60" cy="30" r="2.8" />
                <circle cx="60" cy="30" r="2.8" transform="rotate(72 60 60)" />
                <circle cx="60" cy="30" r="2.8" transform="rotate(144 60 60)" />
                <circle cx="60" cy="30" r="2.8" transform="rotate(216 60 60)" />
                <circle cx="60" cy="30" r="2.8" transform="rotate(288 60 60)" />
              </g>
            </svg>
          </div>
          <Show
            when={!sysStore.loading}
            fallback={
              <div class="skeleton-col">
                <Skeleton class="skeleton-hero-label" />
                <Skeleton class="skeleton-hero-title" />
              </div>
            }
          >
            <div class="hero-content">
              <span class="hero-greeting">Welcome to</span>
              <span class="hero-value">Magic Mount-rs</span>
              <Show
                when={sysStore.device.model && sysStore.device.model !== "-"}
              >
                <span class="hero-subtitle">{sysStore.device.model}</span>
              </Show>
            </div>
          </Show>
        </div>

        <div class="metrics-row">
          <div class="metric-card">
            <Show
              when={moduleStatsReady()}
              fallback={<Skeleton class="skeleton-metric" />}
            >
              <div class="metric-icon-bg">
                <svg viewBox="0 0 24 24">
                  <path d={ICONS.modules} />
                </svg>
              </div>
              <span class="metric-value">{mountedCount()}</span>
              <span class="metric-label">{uiStore.L.status.moduleActive}</span>
            </Show>
          </div>

          <div class="metric-card">
            <Show
              when={!sysStore.loading}
              fallback={<Skeleton class="skeleton-metric" />}
            >
              <div class="metric-icon-bg">
                <svg viewBox="0 0 24 24">
                  <path d={ICONS.ksu} />
                </svg>
              </div>
              <span class="metric-value">{configStore.config.mountsource}</span>
              <span class="metric-label">{uiStore.L.config.mountSource}</span>
            </Show>
          </div>
        </div>

        <div class="info-card">
          <div class="card-title">{uiStore.L.status.sysInfoTitle}</div>

          <div class="info-row">
            <span class="info-key">{uiStore.L.status.kernelLabel}</span>
            <Show
              when={!sysStore.loading}
              fallback={<Skeleton class="skeleton-info-wide" />}
            >
              <span class="info-val">{sysStore.systemInfo.kernel ?? "-"}</span>
            </Show>
          </div>

          <div class="info-row">
            <span class="info-key">{uiStore.L.status.selinuxLabel}</span>
            <Show
              when={!sysStore.loading}
              fallback={<Skeleton class="skeleton-info-narrow" />}
            >
              <span class="info-val">{sysStore.systemInfo.selinux ?? "-"}</span>
            </Show>
          </div>
        </div>
      </div>

      <BottomActions>
        <div class="spacer" />
        <div class="action-row">
          <md-filled-tonal-icon-button
            class="reboot-btn"
            onClick={() => setShowRebootConfirm(true)}
            title={uiStore.L.common.reboot}
          >
            <md-icon>
              <svg viewBox="0 0 24 24">
                <path d={ICONS.power} />
              </svg>
            </md-icon>
          </md-filled-tonal-icon-button>

          <md-filled-tonal-icon-button
            onClick={() => {
              sysStore.loadStatus();
            }}
            disabled={sysStore.loading}
            title={uiStore.L.status.refresh}
          >
            <md-icon>
              <svg viewBox="0 0 24 24">
                <path d={ICONS.refresh} />
              </svg>
            </md-icon>
          </md-filled-tonal-icon-button>
        </div>
      </BottomActions>
    </>
  );
}
