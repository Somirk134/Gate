<!--
  TunnelToolbar — 左栏工具栏
  ------------------------------------------------------------------
  整合 搜索 + 筛选 + 排序 + 新建按钮。
-->
<template>
  <div class="tunnel-list__toolbar">
    <TunnelSearch
      :model-value="query"
      @update:model-value="$emit('update:query', $event)"
    />

    <div class="tunnel-toolbar__row">
      <TunnelFilter
        :model-value="filter"
        :counts="counts"
        @update:model-value="$emit('update:filter', $event)"
      />
      <TunnelSort
        :model-value="sortBy"
        :direction="direction"
        @update:model-value="$emit('update:sortBy', $event)"
        @update:direction="$emit('update:direction', $event)"
      />
    </div>

    <GButton variant="primary" size="sm" icon="plus" block @click="$emit('create')">
      New Tunnel
    </GButton>
  </div>
</template>

<script setup lang="ts">
import GButton from "@components/base/GButton.vue"
import TunnelSearch from "./TunnelSearch.vue"
import TunnelFilter from "./TunnelFilter.vue"
import TunnelSort from "./TunnelSort.vue"
import type { TunnelFilterType, TunnelSortType, SortDirection } from "../types"

defineProps<{
  query: string
  filter: TunnelFilterType
  sortBy: TunnelSortType
  direction: SortDirection
  counts: Record<TunnelFilterType, number>
}>()

defineEmits<{
  "update:query": [value: string]
  "update:filter": [value: TunnelFilterType]
  "update:sortBy": [value: TunnelSortType]
  "update:direction": [value: SortDirection]
  create: []
}>()
</script>

<style scoped>
.tunnel-toolbar__row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-2);
  flex-wrap: wrap;
}
</style>
