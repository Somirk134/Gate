<!--
  ServerToolbar — 左栏工具栏
  ------------------------------------------------------------------
  整合 搜索 + 筛选 + 排序 + 新建按钮。
-->
<template>
  <div class="server-list__toolbar">
    <ServerSearch
      :model-value="query"
      @update:model-value="$emit('update:query', $event)"
    />

    <div class="server-toolbar__row">
      <ServerFilter
        :model-value="filter"
        :counts="counts"
        @update:model-value="$emit('update:filter', $event)"
      />
      <ServerSort
        :model-value="sortBy"
        :direction="direction"
        @update:model-value="$emit('update:sortBy', $event)"
        @update:direction="$emit('update:direction', $event)"
      />
    </div>

    <GButton variant="primary" size="sm" icon="plus" block @click="$emit('create')">
      Add Server
    </GButton>
  </div>
</template>

<script setup lang="ts">
import GButton from "@components/base/GButton.vue"
import ServerSearch from "./ServerSearch.vue"
import ServerFilter from "./ServerFilter.vue"
import ServerSort from "./ServerSort.vue"
import type { ServerFilterType, ServerSortType, SortDirection } from "../types"

defineProps<{
  query: string
  filter: ServerFilterType
  sortBy: ServerSortType
  direction: SortDirection
  counts: Record<ServerFilterType, number>
}>()

defineEmits<{
  "update:query": [value: string]
  "update:filter": [value: ServerFilterType]
  "update:sortBy": [value: ServerSortType]
  "update:direction": [value: SortDirection]
  create: []
}>()
</script>

<style scoped>
.server-toolbar__row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-2);
  flex-wrap: wrap;
}
</style>
