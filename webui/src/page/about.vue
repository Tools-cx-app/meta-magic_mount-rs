<!--

    Copyright (C) 2026 Tools-cx-app <localhost.hutao@gmail.com>
    SPDX-License-Identifier: Apache-2.0

-->
<script setup lang="ts">
import { ref } from "vue";
import {
  MiuixCard,
  MiuixArrowPreference,
  MiuixSmallTitle,
  MiuixBasicComponent,
  MiuixText,
} from "miuix-vue";
import hybrid from "../components/logo.vue";
import { useI18n } from "vue-i18n";
import axios from "axios";
import { API } from "../lib/api";

const version = "v0.1.0-alpha";

interface Contributor {
  id: number;
  login: string;
  name: string;
  url: string;
  html_url: string;
  bio: string;
  type: string;
}

const { t } = useI18n();
const contributors = ref<Contributor[]>([]);

axios
  .get<Contributor[]>(
    "https://api.github.com/repos/tools-cx-app/meta-magic_mount-rs/contributors",
  )
  .then(function (response) {
    console.info(response.data);
    contributors.value = response.data.filter(function (contributor) {
      return contributor.type === "User";
    });
    for (let i = 0; i < contributors.value.length; i++) {
      axios.get(contributors.value[i].url).then(function (response) {
        if (response.data.bio === null) {
          contributors.value[i].bio = t("info.noBio");
        } else {
          contributors.value[i].bio = response.data.bio;
        }
        if (response.data.name === null) {
          contributors.value[i].name = contributors.value[i].login;
        } else {
          contributors.value[i].name = response.data.name;
        }
      });
    }
  })
  .catch(function (error) {
    console.error(error);
    contributors.value = [];
  });

console.info(contributors.value);
function open_github_repo() {
  API.openLink("https://github.com/tools-cx-app/meta-magic_mount-rs");
}
</script>

<template>
  <div class="page">
    <div class="hero">
      <hybrid />
      <h1>{{ t("common.appName") }}</h1>
      <MiuixText>{{ version }}</MiuixText>
    </div>

    <MiuixCard class="ex-card">
      <MiuixArrowPreference
        :title="t('info.projectLink')"
        summary="github.com/tools-cx-app/meta-magic_mount-rs"
        @click="open_github_repo"
      />
    </MiuixCard>

    <MiuixSmallTitle :text="t('info.contributors')" />
    <MiuixCard class="ex-card" :title="contributors.length">
      <div
        v-if="contributors.length > 0"
        v-for="contributor in contributors"
        :key="contributor.id"
      >
        <MiuixBasicComponent
          :title="contributor.name"
          :summary="contributor.bio"
          :clickable="true"
          @click="API.openLink(contributor.html_url)"
        />
      </div>
      <div v-else>
        <MiuixBasicComponent :title="t('info.loadFail')" />
      </div>
    </MiuixCard>
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

.ex-mb12 {
  margin-bottom: 12px;
}

.fade-scale-enter-active,
.fade-scale-leave-active {
  transition: all 0.3s ease;
}
.fade-scale-enter,
.fade-scale-leave-to {
  opacity: 0;
  transform: scale(0.8);
}
.fade-scale-enter-from,
.fade-scale-leave {
  opacity: 0;
  transform: scale(0.8);
}
</style>
