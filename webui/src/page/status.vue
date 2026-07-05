<!--

    Copyright (C) 2026 meta-magic_mount-rs developers
    SPDX-License-Identifier: GPL-v3

-->
<script setup lang="ts">
import { onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { MiuixCard, MiuixSmallTitle, MiuixBasicComponent } from "miuix-vue";
import StatusCard from "../components/miuix/StatusCard.vue";
import { uiStore } from "../lib/stores/uiStore.ts";
import { sysStore } from "../lib/stores/sysStore";
import { moduleStore } from "../lib/stores/moduleStore";
import { configStore } from "../lib/stores/configStore";

const { t } = useI18n();

onMounted(async () => {
  await Promise.all([
    uiStore.init(),
    sysStore.loadStatus(),
    moduleStore.loadModules(),
    configStore.loadConfig(),
  ]);
});
</script>

<template>
  <div class="page">
    <StatusCard
      class="ex-card"
      status="running"
      :label="t('content.welcome')"
      :description="t('content.mmrs')"
      :summary="sysStore.device.model"
    />
    <div class="ex-card-row">
      <MiuixCard class="ex-card--pad ex-grow">
        <MiuixBasicComponent :title="t('status.moduleActive')">
          <template #end>
            <MiuixText>
              {{
                moduleStore.modules.filter((module) => module.is_mounted).length
              }}
            </MiuixText>
          </template>
        </MiuixBasicComponent>
      </MiuixCard>
      <MiuixCard class="ex-card--pad ex-grow">
        <MiuixBasicComponent :title="t('status.mountSource')">
          <template #end>
            <MiuixText>{{ configStore.config.mountsource }}</MiuixText>
          </template>
        </MiuixBasicComponent>
      </MiuixCard>
    </div>
    <MiuixSmallTitle :text="t('status.sysInfoTitle')" />
    <MiuixCard class="ex-card">
      <MiuixBasicComponent
        :title="t('status.kernelLabel')"
        :summary="sysStore.systemInfo.kernel"
      />
      <MiuixBasicComponent
        :title="t('status.selinuxLabel')"
        :summary="sysStore.systemInfo.selinux"
      />
    </MiuixCard>
  </div>
</template>

<style scoped>
.ex-card {
  margin: 0 12px 12px;

  &--pad .m-card {
    padding: 16px;
  }
}
.ex-card-row {
  display: flex;
  gap: 12px;
  margin: 0 12px 12px;
}
.ex-basic-row {
  display: flex;
  gap: 12px;
}
.ex-grow {
  flex: 1;
}
</style>
