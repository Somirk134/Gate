<!--
  ServerSort — 排序选择器
  ------------------------------------------------------------------
  支持：名称 / Ping / CPU / Memory / Tunnel 数 / Project 数 / 地区
  下拉形态，点击切换方向。
-->
<template>
  <div class="server-sort">
    <GIconButton
      name="arrow-up-down"
      size="sm"
      tooltip="切换排序方向"
      @click="toggleDirection"
    />
    <div class="server-sort__select-wrap">
      <select
        :value="modelValue"
        class="server-sort__select"
        @change="onChange"
      >
        <option
          v-for="item in items"
          :key="item.key"
          :value="item.key"
        >
          {{ item.label }}
        </option>
      </select>
      <GIcon
        name="chevron-down"
        :size="12"
        class="server-sort__chevron"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import GIcon from "@components/icons/GIcon.vue"
import GIconButton from "@components/base/GIconButton.vue"
import type { ServerSortType, SortDirection } from "../types"

const props = defineProps<{
  modelValue: ServerSortType
  direction: SortDirection
}>()

const emit = defineEmits<{
  "update:modelValue": [value: ServerSortType]
  "update:direction": [value: SortDirection]
}>()

const items: Array<{ key: ServerSortType; label: string }> = [
  { key: "name", label: "名称" },
  { key: "ping", label: "Ping" },
  { key: "cpu", label: "CPU" },
  { key: "memory", label: "内存" },
  { key: "tunnels", label: "Tunnel 数" },
  { key: "projects", label: "Project 数" },
  { key: "region", label: "地区" },
]

function onChange(e: Event) {
  emit("update:modelValue", (e.target as HTMLSelectElement).value as ServerSortType)
}

function toggleDirection() {
  emit("update:direction", props.direction === "asc" ? "desc" : "asc")
}
</script>
