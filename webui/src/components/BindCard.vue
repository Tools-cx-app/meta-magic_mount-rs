<script setup lang="ts">
import { MiuixText } from 'miuix-vue'
import { Settings } from 'miuix-vue/icons'
import CustomIconButton from './CustomIconButton.vue'

interface Props {
  source: string
  target: string
  sourceLabel?: string
  targetLabel?: string
}

withDefaults(defineProps<Props>(), {
  sourceLabel: 'Source',
  targetLabel: 'Target'
})

const emit = defineEmits<{
  edit: []
}>()
</script>

<template>
  <div class="custom-mount-row">
    <div class="custom-mount-row-content">
      <div class="custom-mount-meta">
        <MiuixText class="custom-mount-label">{{ sourceLabel }}</MiuixText>
        <MiuixText class="custom-mount-value">{{ source }}</MiuixText>
      </div>
      <div class="custom-mount-meta">
        <MiuixText class="custom-mount-label">{{ targetLabel }}</MiuixText>
        <MiuixText class="custom-mount-value">{{ target }}</MiuixText>
      </div>
    </div>
    <CustomIconButton :icon="Settings" :size="24" @click="emit('edit')" />
  </div>
</template>

<style scoped>
.custom-mount-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px;
  background-color: var(--m-color-secondary);
  border-radius: 8px;
  margin-bottom: 8px;
}

.custom-mount-row-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.custom-mount-meta {
  display: flex;
  align-items: center;
  gap: 8px;
}

.custom-mount-label {
  font-size: 12px;
  color: var(--m-color-on-surface-tertiary);
  min-width: 60px;
}

.custom-mount-value {
  font-size: 13px;
  color: var(--m-color-on-surface-tertiary);
  word-break: break-all;
}
</style>
