<template>
  <footer class="log-statusbar">
    <span>{{ filtered }} / {{ total }} {{ t('logs.entries') }}</span>
    <span>{{ paused ? t('logs.paused') : t('logs.runtime') }}</span>
    <span
      >{{ t('logs.autoScroll') }}:
      {{ autoScroll ? t('logs.autoScrollOn') : t('logs.autoScrollOff') }}</span
    >
    <span v-if="dropped">{{ t('logs.dropped') }}: {{ dropped }}</span>
    <span v-if="selected">{{ t('logs.selected') }}: {{ selected }}</span>
  </footer>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'

defineProps<{
  total: number
  filtered: number
  paused: boolean
  autoScroll: boolean
  dropped: number
  selected?: string
}>()

const { t } = useI18n()
</script>
