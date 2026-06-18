<script setup lang="ts">
import { MiuixText } from 'miuix-vue'
import { Close } from 'miuix-vue/icons'
import CustomIconButton from './CustomIconButton.vue'

interface Props {
  text: string
  buttonalign?: string
}

const props = withDefaults(defineProps<Props>(), {
  buttonalign: 'end',
})

const emit = defineEmits<{
  remove: []
}>()
</script>

<template>
  <div class="chip-item">
    <CustomIconButton v-if="props.buttonalign === 'start'" :icon="Close" :size="16" @click="emit('remove')" />
    <MiuixText size="13">{{ text }}</MiuixText>
    <CustomIconButton  v-if="props.buttonalign === 'end'" :icon="Close" :size="16" @click="emit('remove')" />
  </div>
</template>

<style scoped>
.chip-item {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 8px;
  background-color: var(--m-color-secondary-container);
  border-radius: 16px;
  margin-right: 8px;
  margin-bottom: 8px;
}

</style>
