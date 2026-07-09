<template>
  <aside class="settings-inspector">
    <header class="settings-inspector__header">
      <GIcon name="circle-help" :size="16" />
      <span>说明</span>
    </header>

    <SettingDescription
      v-if="context"
      :context="context"
      :current-value="currentValue"
      :modified="modified"
      :error="error" />

    <div v-else class="settings-inspector__empty">
      <GIcon name="settings" :size="28" />
      <p>选择一个设置项</p>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import GIcon from '@components/icons/GIcon.vue'
import type { SettingContext, SettingValue } from '../types'
import SettingDescription from './SettingDescription.vue'

const props = defineProps<{
  context: SettingContext | null
  values: Record<string, SettingValue>
  dirtyKeys: string[]
  validationErrors: Record<string, string | undefined>
}>()

const currentValue = computed(() => {
  if (!props.context) return null
  return props.values[props.context.item.key]
})
const modified = computed(() =>
  props.context ? props.dirtyKeys.includes(props.context.item.key) : false,
)
const error = computed(() =>
  props.context ? props.validationErrors[props.context.item.key] : undefined,
)
</script>
