<!--
  TunnelSort — 排序选择器
  ------------------------------------------------------------------
  支持：名称 / 状态 / 流量 / 连接数 / 创建时间 / 更新时间
  下拉形态，点击切换方向。
-->
<template>
  <div class="tunnel-sort">
    <GIconButton
      name="arrow-up-down"
      size="sm"
      :tooltip="t('tunnel.sort.directionTooltip')"
      @click="toggleDirection" />
    <div class="tunnel-sort__select-wrap">
      <select :value="modelValue" class="tunnel-sort__select" @change="onChange">
        <option v-for="item in items" :key="item.key" :value="item.key">
          {{ item.label }}
        </option>
      </select>
      <GIcon name="chevron-down" :size="12" class="tunnel-sort__chevron" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import GIcon from '@components/icons/GIcon.vue'
import GIconButton from '@components/base/GIconButton.vue'
import type { TunnelSortType, SortDirection } from '../types'

const props = defineProps<{
  modelValue: TunnelSortType
  direction: SortDirection
}>()

const emit = defineEmits<{
  'update:modelValue': [value: TunnelSortType]
  'update:direction': [value: SortDirection]
}>()

const { t } = useI18n()

const items = computed<Array<{ key: TunnelSortType; label: string }>>(() => [
  { key: 'name', label: t('tunnel.sort.name') },
  { key: 'status', label: t('tunnel.sort.status') },
  { key: 'traffic', label: t('tunnel.sort.traffic') },
  { key: 'connections', label: t('tunnel.sort.connections') },
  { key: 'createdAt', label: t('tunnel.sort.createdAt') },
  { key: 'updatedAt', label: t('tunnel.sort.updatedAt') },
])

function onChange(e: Event) {
  emit('update:modelValue', (e.target as HTMLSelectElement).value as TunnelSortType)
}

function toggleDirection() {
  emit('update:direction', props.direction === 'asc' ? 'desc' : 'asc')
}
</script>
