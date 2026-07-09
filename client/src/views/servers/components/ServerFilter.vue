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
      @click="$emit('update:modelValue', item.key)"
    >
      <GIcon
        v-if="item.icon"
        :name="item.icon"
        :size="12"
      />
      <span>{{ item.label }}</span>
      <span class="server-segment__count">{{ counts[item.key] ?? 0 }}</span>
    </button>
  </div>
</template>

<script setup lang="ts">
import GIcon from "@components/icons/GIcon.vue"
import type { ServerFilterType } from "../types"

defineProps<{
  modelValue: ServerFilterType
  counts: Record<ServerFilterType, number>
}>()

defineEmits<{ "update:modelValue": [value: ServerFilterType] }>()

const items: Array<{ key: ServerFilterType; label: string; icon?: string }> = [
  { key: "all", label: "全部" },
  { key: "online", label: "在线", icon: "wifi" },
  { key: "offline", label: "离线", icon: "wifi-off" },
  { key: "favorite", label: "收藏", icon: "star" },
  { key: "recent", label: "最近", icon: "clock" },
  { key: "unhealthy", label: "异常", icon: "alert-triangle" },
]
</script>
