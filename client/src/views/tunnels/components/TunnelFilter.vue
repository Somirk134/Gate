<!--
  TunnelFilter — 分段筛选器
  ------------------------------------------------------------------
  支持：全部 / HTTP / TCP / 运行中 / 已停止 / 收藏 / 最近
  分段控件形态，每项显示计数。
-->
<template>
  <div class="tunnel-segment">
    <button
      v-for="item in items"
      :key="item.key"
      type="button"
      class="tunnel-segment__item"
      :class="{ 'tunnel-segment__item--active': modelValue === item.key }"
      @click="$emit('update:modelValue', item.key)">
      <GIcon v-if="item.icon" :name="item.icon" :size="12" />
      <span>{{ item.label }}</span>
      <span class="tunnel-segment__count">{{ counts[item.key] ?? 0 }}</span>
    </button>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import GIcon from '@components/icons/GIcon.vue'
import type { TunnelFilterType } from '../types'

defineProps<{
  modelValue: TunnelFilterType
  counts: Record<TunnelFilterType, number>
}>()

defineEmits<{ 'update:modelValue': [value: TunnelFilterType] }>()

const { t } = useI18n()

const items = computed<Array<{ key: TunnelFilterType; label: string; icon?: string }>>(() => [
  { key: 'all', label: t('tunnel.filters.all') },
  { key: 'http', label: 'HTTP', icon: 'globe' },
  { key: 'tcp', label: 'TCP', icon: 'router' },
  { key: 'running', label: t('tunnel.filters.running'), icon: 'play' },
  { key: 'stopped', label: t('tunnel.filters.stopped'), icon: 'stop' },
  { key: 'favorite', label: t('tunnel.filters.favorite'), icon: 'star' },
  { key: 'recent', label: t('tunnel.filters.recent'), icon: 'clock' },
])
</script>
