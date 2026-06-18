<script setup lang="ts">
import { onMounted } from "vue";
import { useI18n } from "vue-i18n";
import {
  MiuixCard,
  MiuixSmallTitle,
  MiuixBasicComponent,
  MiuixIcon,
} from "miuix-vue";
import { Info } from "miuix-vue/icons";
import { sysStore } from "../lib/stores/sysStore";
import { moduleStore } from "../lib/stores/moduleStore";
import { configStore } from "../lib/stores/configStore";

const { t } = useI18n();

onMounted(async () => {
  await Promise.all([
    sysStore.loadStatus(),
    moduleStore.loadModules(),
    configStore.loadConfig(),
  ]);
});
</script>

<template>
  <div class="page">
    <MiuixCard
      class="ex-card ex-card--pad"
      show-indication
      style="--m-card-color: var(--m-color-primary-variant)"
    >
      <MiuixBasicComponent
        :title="t('content.welcome')"
        titleColor="var(--m-color-on-primary-variant)"
        :summary="sysStore.device.model"
        summaryColor="var(--m-color-on-primary-variant)"
      >
        <template #start>
          <MiuixIcon color="var(--m-color-on-primary-variant)" :icon="Info" />
        </template>
        <template #end>
          <MiuixText style="color: var(--m-color-on-primary-variant)">
            <b>{{ t("content.mmrs") }}</b>
          </MiuixText>
        </template>
      </MiuixBasicComponent>
    </MiuixCard>
    <MiuixCard class="ex-card">
      <div class="ex-basic-row ex-grow">
        <MiuixBasicComponent :title="t('status.moduleActive')">
          <template #end>
            <MiuixText>{{ moduleStore.modules.length }}</MiuixText>
          </template>
        </MiuixBasicComponent>
        <MiuixBasicComponent :title="t('config.mountSource')">
          <template #end>
            <MiuixText>{{ configStore.config.mountsource }}</MiuixText>
          </template>
        </MiuixBasicComponent>
      </div>
    </MiuixCard>

    <MiuixSmallTitle :text="t('status.sysInfoTitle')" />
    <MiuixCard class="ex-card">
      <MiuixBasicComponent
        :title="t('status.kernel')"
        :summary="sysStore.systemInfo.kernel"
      />
      <MiuixBasicComponent
        :title="t('status.selinux')"
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
