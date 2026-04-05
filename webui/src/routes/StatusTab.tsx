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

const STAR_PATH =
  "M12 2.25c.19 5.34 4.41 9.56 9.75 9.75-5.34.19-9.56 4.41-9.75 9.75-.19-5.34-4.41-9.56-9.75-9.75C7.59 11.81 11.81 7.59 12 2.25z";

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
            <svg viewBox="0 0 24 24">
              <path d={STAR_PATH} />
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
