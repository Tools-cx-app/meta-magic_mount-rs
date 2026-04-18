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

import { For, Show, createSignal, onCleanup, onMount } from "solid-js";

import MagicLogo from "../components/MagicLogo";
import Skeleton from "../components/Skeleton";
import { API } from "../lib/api";
import { ICONS } from "../lib/constants";
import { sysStore } from "../lib/stores/sysStore";
import { uiStore } from "../lib/stores/uiStore";

import "@material/web/button/filled-tonal-button.js";
import "@material/web/icon/icon.js";
import "@material/web/list/list-item.js";
import "@material/web/list/list.js";
import "./InfoTab.css";

const REPO_OWNER = "Tools-cx-app";
const REPO_NAME = "meta-magic_mount-rs";
const CACHE_KEY = "mm_contributors_cache";
const CACHE_DURATION = 1000 * 60 * 60;
const DETAIL_FETCH_LIMIT = 12;

interface Contributor {
  login: string;
  avatar_url: string;
  html_url: string;
  type: string;
  url: string;
  name?: string;
  bio?: string;
}

interface ContributorCache {
  data: Contributor[];
  timestamp: number;
}

export default function InfoTab() {
  const [contributors, setContributors] = createSignal<Contributor[]>([]);
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal(false);
  const controller = new AbortController();

  onMount(() => {
    void fetchContributors();
  });

  onCleanup(() => controller.abort());

  async function fetchContributors() {
    const cached = localStorage.getItem(CACHE_KEY);

    if (cached) {
      try {
        const { data, timestamp } = JSON.parse(cached) as ContributorCache;

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
      const response = await fetch(
        `https://api.github.com/repos/${REPO_OWNER}/${REPO_NAME}/contributors`,
        { signal: controller.signal },
      );

      if (!response.ok) {
        throw new Error("Failed to fetch contributors");
      }

      const basicList = (await response.json()) as Contributor[];
      const filteredList = basicList.filter((user) => {
        const isBotType = user.type === "Bot";
        const hasBotName = user.login.toLowerCase().includes("bot");

        return !isBotType && !hasBotName;
      });

      const enriched = [...filteredList];
      const detailTargets = filteredList.slice(0, DETAIL_FETCH_LIMIT);
      const detailResults = await Promise.allSettled(
        detailTargets.map(async (user) => {
          const detailResponse = await fetch(user.url, {
            signal: controller.signal,
          });

          if (!detailResponse.ok) {
            throw new Error(`Failed to fetch ${user.login}`);
          }

          const detail = await detailResponse.json();

          return {
            ...user,
            bio: detail.bio ?? user.bio,
            name: detail.name ?? user.login,
          } as Contributor;
        }),
      );

      for (const [index, result] of detailResults.entries()) {
        if (result.status === "fulfilled") {
          enriched[index] = result.value;
        }
      }

      setContributors(enriched);
      localStorage.setItem(
        CACHE_KEY,
        JSON.stringify({
          data: enriched,
          timestamp: Date.now(),
        }),
      );
    } catch (error) {
      if ((error as Error).name !== "AbortError") {
        setError(true);
      }
    } finally {
      setLoading(false);
    }
  }

  function handleLink(event: MouseEvent, url: string) {
    event.preventDefault();
    void API.openLink(url);
  }

  return (
    <div class="info-container">
      <div class="project-header">
        <div class="app-logo">
          <MagicLogo />
        </div>
        <span class="app-name">{uiStore.L.common.appName}</span>
        <span class="app-version">{sysStore.version}</span>
      </div>

      <div class="action-buttons">
        <md-filled-tonal-button
          class="action-btn action-btn-wide"
          onClick={(event: MouseEvent) =>
            handleLink(event, `https://github.com/${REPO_OWNER}/${REPO_NAME}`)
          }
        >
          <md-icon slot="icon">
            <svg viewBox="0 0 24 24">
              <path d={ICONS.github} />
            </svg>
          </md-icon>
          {uiStore.L.info.projectLink}
        </md-filled-tonal-button>
      </div>

      <div class="contributors-section">
        <div class="section-title">{uiStore.L.info.contributors}</div>

        <div class="list-wrapper">
          <Show
            when={!loading()}
            fallback={
              <For each={Array.from({ length: 3 })}>
                {() => (
                  <div class="skeleton-item">
                    <Skeleton class="skeleton-avatar" />
                    <div class="skeleton-text">
                      <Skeleton class="skeleton-text-title" />
                      <Skeleton class="skeleton-text-body" />
                    </div>
                  </div>
                )}
              </For>
            }
          >
            <Show
              when={!error()}
              fallback={
                <div class="error-message">{uiStore.L.info.loadFail}</div>
              }
            >
              <md-list class="contributors-list">
                <For each={contributors()}>
                  {(user) => (
                    <md-list-item
                      class="contributor-link"
                      type="link"
                      href={user.html_url}
                      target="_blank"
                      onClick={(event: MouseEvent) =>
                        handleLink(event, user.html_url)
                      }
                    >
                      <img
                        slot="start"
                        src={`${user.avatar_url}${user.avatar_url.includes("?") ? "&" : "?"}s=80`}
                        alt={user.login}
                        class="c-avatar"
                        loading="lazy"
                      />
                      <div slot="headline">{user.name ?? user.login}</div>
                      <div slot="supporting-text">
                        {user.bio ?? uiStore.L.info.noBio}
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
