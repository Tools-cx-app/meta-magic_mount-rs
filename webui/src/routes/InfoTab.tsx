/**
 * Copyright 2025 Magic Mount-rs Authors
 *
 * SPDX-License-Identifier: GPL-3.0-or-later
 */

import { For, Show, createSignal, onMount } from "solid-js";

import MagicLogo from "../components/MagicLogo";
import Skeleton from "../components/Skeleton";
import { API } from "../lib/api";
import { ICONS } from "../lib/constants";
import { store } from "../lib/store";

import "./InfoTab.css";
import "@material/web/button/filled-tonal-button.js";
import "@material/web/icon/icon.js";
import "@material/web/list/list.js";
import "@material/web/list/list-item.js";

const REPO_OWNER = "Tools-cx-app";
const REPO_NAME = "meta-magic_mount-rs";
const CACHE_KEY = "mm_contributors_cache";
const CACHE_DURATION = 1000 * 60 * 60;

interface Contributor {
  login: string;
  id: number;
  avatar_url: string;
  html_url: string;
  type: string;
  name?: string;
  bio?: string;
  url?: string;
  [key: string]: any;
}

export default function InfoTab() {
  const [contributors, setContributors] = createSignal<Contributor[]>([]);
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal(false);
  const [version, setVersion] = createSignal(store.version);

  onMount(async () => {
    try {
      const v = await API.getVersion();
      if (v) {
        setVersion(v);
      }
    } catch (e) {
      console.error("Failed to fetch version", e);
    }
    await fetchContributors();
  });

  async function fetchContributors() {
    const cached = localStorage.getItem(CACHE_KEY);
    if (cached) {
      try {
        const { data, timestamp } = JSON.parse(cached);
        if (Date.now() - timestamp < CACHE_DURATION) {
          setContributors(data);
          setLoading(false);

          return;
        }
      } catch {
        localStorage.removeItem(CACHE_KEY);
      }
    }

    try {
      const res = await fetch(
        `https://api.github.com/repos/${REPO_OWNER}/${REPO_NAME}/contributors`,
      );
      if (!res.ok) {
        throw new Error("Failed to fetch list");
      }

      const basicList: Contributor[] = await res.json();
      const filteredList = basicList.filter((user) => {
        const isBotType = user.type === "Bot";
        const hasBotName = user.login.toLowerCase().includes("bot");

        return !isBotType && !hasBotName;
      });

      const detailPromises = filteredList.map(async (user) => {
        try {
          const detailRes = await fetch(user.url!);
          if (detailRes.ok) {
            const detail = await detailRes.json();

            return {
              ...user,
              bio: detail.bio,
              name: detail.name ?? user.login,
            };
          }
        } catch {
          console.warn("Failed to fetch detail for", user.login);
        }

        return user;
      });
      const result = await Promise.all(detailPromises);
      setContributors(result);

      localStorage.setItem(
        CACHE_KEY,
        JSON.stringify({
          data: result,
          timestamp: Date.now(),
        }),
      );
    } catch (e) {
      console.error(e);
      setError(true);
    } finally {
      setLoading(false);
    }
  }

  function handleLink(e: Event, url: string) {
    e.preventDefault();
    API.openLink(url);
  }

  return (
    <div class="info-container">
      <div class="project-header">
        <div class="app-logo">
          <MagicLogo />
        </div>
        <span class="app-name">{store.L.common.appName}</span>
        <span class="app-version">{version()}</span>
      </div>

      <div class="action-buttons">
        <md-filled-tonal-button
          class="action-btn"
          on:click={(e) =>
            handleLink(e, `https://github.com/${REPO_OWNER}/${REPO_NAME}`)
          }
        >
          <md-icon slot="icon">
            <svg viewBox="0 0 24 24">
              <path d={ICONS.github} />
            </svg>
          </md-icon>
          {store.L.info.projectLink}
        </md-filled-tonal-button>
      </div>

      <div class="contributors-section">
        <div class="section-title">{store.L.info.contributors}</div>
        <div class="list-wrapper">
          <Show
            when={!loading()}
            fallback={
              <For each={Array.from({ length: 3 })}>
                {() => (
                  <div class="skeleton-item">
                    <Skeleton width="40px" height="40px" borderRadius="50%" />
                    <div class="skeleton-text">
                      <Skeleton width="120px" height="16px" />
                      <Skeleton width="180px" height="12px" />
                    </div>
                  </div>
                )}
              </For>
            }
          >
            <Show
              when={!error()}
              fallback={
                <div class="error-message">{store.L.info.loadFail}</div>
              }
            >
              <md-list class="contributors-list">
                <For each={contributors()}>
                  {(user) => (
                    <md-list-item
                      prop:type="link"
                      prop:href={user.html_url}
                      prop:target="_blank"
                      on:click={(e: MouseEvent) => handleLink(e, user.html_url)}
                    >
                      <img
                        slot="start"
                        src={user.avatar_url}
                        alt={user.login}
                        class="c-avatar"
                        loading="lazy"
                      />
                      <div slot="headline">{user.name ?? user.login}</div>
                      <div slot="supporting-text">
                        {user.bio ?? store.L.info.noBio}
                      </div>
                    </md-list-item>
                  )}
                </For>
              </md-list>
            </Show>
          </Show>
        </div>
      </div>
    </div>
  );
}
