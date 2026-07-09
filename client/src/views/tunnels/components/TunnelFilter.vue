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
      @click="$emit('update:modelValue', item.key)"
    >
      <GIcon
        v-if="item.icon"
        :name="item.icon"
        :size="12"
      />
      <span>{{ item.label }}</span>
      <span class="tunnel-segment__count">{{ counts[item.key] ?? 0 }}</span>
    </button>
  </div>
</template>

<script setup lang="ts">
import GIcon from "@components/icons/GIcon.vue"
import type { TunnelFilterType } from "../types"

defineProps<{
  modelValue: TunnelFilterType
  counts: Record<TunnelFilterType, number>
}>()

defineEmits<{ "update:modelValue": [value: TunnelFilterType] }>()

const items: Array<{ key: TunnelFilterType; label: string; icon?: string }> = [
  { key: "all", label: "全部" },
  { key: "http", label: "HTTP", icon: "globe" },
  { key: "tcp", label: "TCP", icon: "router" },
  { key: "running", label: "运行中", icon: "play" },
  { key: "stopped", label: "已停止", icon: "stop" },
  { key: "favorite", label: "收藏", icon: "star" },
  { key: "recent", label: "最近", icon: "clock" },
]
</script>
