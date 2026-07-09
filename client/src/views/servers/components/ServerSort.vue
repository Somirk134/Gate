<!--
  ServerSort — 排序选择器
  ------------------------------------------------------------------
  支持：名称 / Ping / CPU / 内存 / 隧道数 / 项目数 / 地区
  下拉形态，点击切换方向。
-->
<template>
  <div class="server-sort">
    <GIconButton
      name="arrow-up-down"
      size="sm"
      :tooltip="t('server.sort.directionTooltip')"
      @click="toggleDirection" />
    <div class="server-sort__select-wrap">
      <select :value="modelValue" class="server-sort__select" @change="onChange">
        <option v-for="item in items" :key="item.key" :value="item.key">
          {{ item.label }}
        </option>
      </select>
      <GIcon name="chevron-down" :size="12" class="server-sort__chevron" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import GIcon from '@components/icons/GIcon.vue'
import GIconButton from '@components/base/GIconButton.vue'
import type { ServerSortType, SortDirection } from '../types'

const props = defineProps<{
  modelValue: ServerSortType
  direction: SortDirection
}>()

const emit = defineEmits<{
  'update:modelValue': [value: ServerSortType]
  'update:direction': [value: SortDirection]
}>()

const { t } = useI18n()

const items = computed<Array<{ key: ServerSortType; label: string }>>(() => [
  { key: 'name', label: t('server.sort.name') },
  { key: 'ping', label: t('server.sort.ping') },
  { key: 'cpu', label: t('server.sort.cpu') },
  { key: 'memory', label: t('server.sort.memory') },
  { key: 'tunnels', label: t('server.sort.tunnels') },
  { key: 'projects', label: t('server.sort.projects') },
  { key: 'region', label: t('server.sort.region') },
])

function onChange(e: Event) {
  emit('update:modelValue', (e.target as HTMLSelectElement).value as ServerSortType)
}

function toggleDirection() {
  emit('update:direction', props.direction === 'asc' ? 'desc' : 'asc')
}
</script>
