/**
 * Copyright 2025 Magic Mount-rs Authors
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import { Show, createMemo, createSignal, onMount } from "solid-js";

import BottomActions from "../components/BottomActions";
import Skeleton from "../components/Skeleton";
import { ICONS } from "../lib/constants";
import { store } from "../lib/store";

import "./StatusTab.css";
import "@material/web/icon/icon.js";
import "@material/web/iconbutton/icon-button.js";
import "@material/web/button/filled-tonal-button.js";
import "@material/web/iconbutton/filled-tonal-icon-button.js";
import "@material/web/dialog/dialog.js";
import "@material/web/button/text-button.js";

export default function StatusTab() {
  onMount(() => {
    store.loadStatus();
  });

  // function copyDebugInfo() {
  //   const info =
  //     `Magic Mount v${store.version}\n` +
  //     `Model: ${store.device.model}\n` +
  //     `Android: ${store.device.android}\n` +
  //     `Kernel: ${store.device.kernel}\n` +
  //     `SELinux: ${store.device.selinux}`;
  //   navigator.clipboard.writeText(info);
  // }

  const [showRebootConfirm, setShowRebootConfirm] = createSignal(false);

  function handleReboot() {
    setShowRebootConfirm(false);
    store.rebootDevice();
  }

  const mountedCount = createMemo(
    () => store.modules?.filter((m) => m.is_mounted).length ?? 0,
  );
  const isSelinuxEnforcing = createMemo(
    () => store.device.selinux === "Enforcing",
  );

  return (
    <>
      <md-dialog
        prop:open={showRebootConfirm()}
        on:close={() => setShowRebootConfirm(false)}
        style={{
          "--md-dialog-scrim-color": "transparent",
          "--md-sys-color-scrim": "transparent",
        }}
      >
        <div slot="headline">{store.L.common.rebootTitle}</div>
        <div slot="content">{store.L.common.rebootConfirm}</div>
        <div slot="actions">
          <md-text-button on:click={() => setShowRebootConfirm(false)}>
            {store.L.common.cancel}
          </md-text-button>
          <md-text-button on:click={handleReboot}>
            {store.L.common.reboot}
          </md-text-button>
        </div>
      </md-dialog>

      <div class="dashboard-grid">
        <div class="hero-card">
          <div class="hero-decoration">
            <svg viewBox="0 0 24 24" fill="currentColor">
              <path d="M7.5 5.6L10 0l2.5 5.6L18 8l-5.5 2.4L10 16l-2.5-5.6L2 8l5.5-2.4zm12 9.4l1.5-3 1.5 3 3 1.5-3 1.5-1.5 3-1.5-3-3-1.5 3-1.5zm-3-8l1.25-2.5 1.25 2.5 2.5 1.25-2.5 1.25-1.25 2.5-1.25-2.5-2.5-1.25 2.5-1.25z" />
            </svg>
          </div>

          <div class="hero-content">
            <div class="hero-label-group">
              <div class="hero-icon-circle">
                <md-icon>
                  <svg viewBox="0 0 24 24">
                    <path d={ICONS.home} />
                  </svg>
                </md-icon>
              </div>
              <span class="hero-title">{store.L.status.deviceTitle}</span>
            </div>
            <div class="hero-main-info">
              <Show
                when={!store.loading.status}
                fallback={
                  <>
                    <Skeleton width="150px" height="32px" />
                    <Skeleton width="80px" height="24px" />
                  </>
                }
              >
                <span class="device-model">{store.device.model}</span>
                <div class="version-pill">
                  <span>Magic Mount v{store.version}</span>
                </div>
              </Show>
            </div>
          </div>
          <div class="hero-actions" />
        </div>

        <div class="stats-row">
          <div class="stat-card">
            <Show
              when={!store.loading.status}
              fallback={
                <>
                  <Skeleton width="40px" height="32px" />
                  <Skeleton width="60px" height="12px" />
                </>
              }
            >
              <div class="stat-value">{mountedCount()}</div>
              <div class="stat-label">{store.L.status.moduleActive}</div>
            </Show>
          </div>

          <div class="stat-card">
            <Show
              when={!store.loading.status}
              fallback={
                <>
                  <Skeleton width="40px" height="32px" />
                  <Skeleton width="60px" height="12px" />
                </>
              }
            >
              <div class="stat-value">{store.config?.mountsource ?? "-"}</div>
              <div class="stat-label">{store.L.config.mountSource}</div>
            </Show>
          </div>
        </div>

        <div class="details-card">
          <div class="card-title">{store.L.status.sysInfoTitle}</div>
          <div class="info-list">
            <div class="info-item">
              <span class="info-label">{store.L.status.androidLabel}</span>
              <Show
                when={!store.loading.status}
                fallback={<Skeleton width="60px" height="16px" />}
              >
                <span class="info-val">{store.device.android}</span>
              </Show>
            </div>

            <div class="info-item">
              <span class="info-label">{store.L.status.selinuxLabel}</span>
              <Show
                when={!store.loading.status}
                fallback={<Skeleton width="80px" height="16px" />}
              >
                <span class={`info-val ${isSelinuxEnforcing() ? "" : "warn"}`}>
                  {store.device.selinux}
                </span>
              </Show>
            </div>

            <div class="info-item full-width">
              <span class="info-label">{store.L.status.kernelLabel}</span>
              <Show
                when={!store.loading.status}
                fallback={<Skeleton width="100%" height="16px" />}
              >
                <span class="info-val mono">{store.device.kernel}</span>
              </Show>
            </div>
          </div>
        </div>
      </div>

      <BottomActions>
        <div class="spacer" />
        <md-filled-tonal-icon-button
          class="reboot-btn"
          on:click={() => setShowRebootConfirm(true)}
          prop:title={store.L.common.reboot}
        >
          <md-icon>
            <svg viewBox="0 0 24 24">
              <path d="M13 3h-2v10h2V3zm4.83 2.17l-1.42 1.42C17.99 7.86 19 9.81 19 12c0 3.87-3.13 7-7 7s-7-3.13-7-7c0-2.19 1.01-4.14 2.58-5.42L6.17 5.17C4.23 6.82 3 9.26 3 12c0 4.97 4.03 9 9 9s9-4.03 9-9c0-2.74-1.23-5.18-3.17-6.83z" />
            </svg>
          </md-icon>
        </md-filled-tonal-icon-button>

        <md-filled-tonal-icon-button
          on:click={() => store.loadStatus()}
          prop:disabled={store.loading.status}
          prop:title={store.L.status.refresh}
        >
          <md-icon>
            <svg viewBox="0 0 24 24">
              <path d={ICONS.refresh} />
            </svg>
          </md-icon>
        </md-filled-tonal-icon-button>
      </BottomActions>
    </>
  );
}
