<!--
  ServerFilter — 分段筛选器
  ------------------------------------------------------------------
  支持：全部 / 在线 / 离线 / 收藏 / 最近 / 健康异常
  分段控件形态，每项显示计数。
-->
<template>
  <div class="server-segment">
    <button
      v-for="item in items"
      :key="item.key"
      type="button"
      class="server-segment__item"
      :class="{ 'server-segment__item--active': modelValue === item.key }"
      @click="$emit('update:modelValue', item.key)">
      <GIcon v-if="item.icon" :name="item.icon" :size="12" />
      <span>{{ item.label }}</span>
      <span class="server-segment__count">{{ counts[item.key] ?? 0 }}</span>
    </button>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import GIcon from '@components/icons/GIcon.vue'
import type { ServerFilterType } from '../types'

defineProps<{
  modelValue: ServerFilterType
  counts: Record<ServerFilterType, number>
}>()

defineEmits<{ 'update:modelValue': [value: ServerFilterType] }>()

const { t } = useI18n()

const items = computed<Array<{ key: ServerFilterType; label: string; icon?: string }>>(() => [
  { key: 'all', label: t('server.filters.all') },
  { key: 'online', label: t('server.filters.online'), icon: 'wifi' },
  { key: 'offline', label: t('server.filters.offline'), icon: 'wifi-off' },
  { key: 'favorite', label: t('server.filters.favorite'), icon: 'star' },
  { key: 'recent', label: t('server.filters.recent'), icon: 'clock' },
  { key: 'unhealthy', label: t('server.filters.unhealthy'), icon: 'alert-triangle' },
])
</script>
