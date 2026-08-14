<!--

    Copyright (C) 2026 meta-magic_mount-rs developers
    SPDX-License-Identifier: GPL-v3

-->
<script setup lang="ts">
import { ref } from "vue";
import {
  MiuixCard,
  MiuixArrowPreference,
  MiuixSmallTitle,
  MiuixBasicComponent,
  MiuixText,
  MiuixProgressIndicator,
} from "miuix-vue";
import magicmount from "../components/logo.vue";
import { useI18n } from "vue-i18n";
import axios from "axios";
import { API } from "../../../lib/api";

interface Contributor {
  id: number;
  login: string;
  name: string;
  url: string;
  html_url: string;
  avatar_url: string;
  bio: string;
  type: string;
}

const version = ref("");

const { t } = useI18n();
const contributors = ref<Contributor[]>([]);
const loading = ref(true);
const error = ref(false);

API.getVersion().then((ver) => {
  version.value = ver;
});

const STORAGE_KEY = "mmrs_contributors";

const cached = localStorage.getItem(STORAGE_KEY);
if (cached) {
  try {
    const parsed = JSON.parse(cached);
    if (
      parsed.timestamp &&
      Date.now() - parsed.timestamp < 24 * 60 * 60 * 1000
    ) {
      contributors.value = parsed.data;
      loading.value = false;
    } else {
      localStorage.removeItem(STORAGE_KEY);
    }
  } catch {
    localStorage.removeItem(STORAGE_KEY);
  }
}

if (!contributors.value.length) {
  axios
    .get<Contributor[]>(
      "https://api.github.com/repos/Tools-cx-app/meta-magic_mount-rs/contributors",
    )
    .then(function (response) {
      const userContributors = response.data.filter(function (contributor) {
        return contributor.type === "User";
      });
      const detailPromises = userContributors.map(function (contributor) {
        return axios
          .get(contributor.url)
          .then(function (detailResponse) {
            return {
              ...contributor,
              bio: detailResponse.data.bio ?? null,
              name: detailResponse.data.name ?? contributor.login,
            };
          })
          .catch(function () {
            return {
              ...contributor,
              bio: null,
              name: contributor.login,
            };
          });
      });
      return Promise.all(detailPromises);
    })
    .then(function (result) {
      contributors.value = result.map((item) => ({
        ...item,
        bio: item.bio ?? null,
      }));
      localStorage.setItem(
        STORAGE_KEY,
        JSON.stringify({
          timestamp: Date.now(),
          data: result,
        }),
      );
      loading.value = false;
    })
    .catch(function () {
      error.value = true;
      loading.value = false;
    });
}

function open_github_repo() {
  API.openLink("https://github.com/Tools-cx-app/meta-magic_mount-rs");
}
</script>

<template>
  <div class="page">
    <div class="hero">
      <magicmount />
      <h1>{{ t("common.appName") }}</h1>
      <MiuixText color="var(--m-color-on-background)">{{ version }}</MiuixText>
    </div>

    <MiuixCard class="ex-card">
      <MiuixArrowPreference
        :title="t('info.projectLink')"
        summary="github.com/Tools-cx-app/meta-magic_mount-rs"
        @click="open_github_repo"
      />
    </MiuixCard>

    <MiuixSmallTitle :text="t('info.contributors')" />
    <MiuixCard v-if="!loading" class="ex-card">
      <div
        v-if="contributors.length > 0"
        v-for="contributor in contributors"
        :key="contributor.id"
      >
        <MiuixBasicComponent
          :title="contributor.name ?? contributor.login"
          :summary="contributor.bio ?? t('info.noBio')"
          :clickable="true"
          @click="API.openLink(contributor.html_url)"
        >
          <template #start>
            <img
              :src="contributor.avatar_url"
              width="42"
              style="margin: 0 8px; border-radius: 12px"
            />
          </template>
        </MiuixBasicComponent>
      </div>
      <div v-else-if="error">
        <MiuixBasicComponent :title="t('info.loadFail')" />
      </div>
    </MiuixCard>
    <div v-else align="center">
      <MiuixProgressIndicator
        type="infinite"
        color="var(--m-color-on-background-variant)"
        style="padding-top: 12px"
      />
    </div>
  </div>
</template>

<style scoped>
.hero {
  position: relative;
  text-align: center;
  padding: 32px 0;

  .base {
    width: 170px;
    position: relative;
    z-index: 0;
  }

  h1 {
    font-size: 56px;
    letter-spacing: -1.68px;
    margin: 32px 0;
    font-weight: 500;
    color: var(--m-color-on-background);

    @media (max-width: 1024px) {
      font-size: 36px;
      margin: 20px 0;
    }
  }
}

.ex-card {
  margin: 0 12px 12px;
}
</style>
