<!--
  ProjectFilter — 分段筛选器
  ------------------------------------------------------------------
  支持：全部 / 运行中 / 已停止 / 收藏 / 最近使用
  分段控件形态，每项显示计数。
-->
<template>
  <div class="projects-segment">
    <button
      v-for="item in items"
      :key="item.key"
      type="button"
      class="projects-segment__item"
      :class="{ 'projects-segment__item--active': modelValue === item.key }"
      @click="$emit('update:modelValue', item.key)"
    >
      <GIcon v-if="item.icon" :name="item.icon" :size="13" />
      <span>{{ item.label }}</span>
      <span class="projects-segment__count">{{ counts[item.key] ?? 0 }}</span>
    </button>
  </div>
</template>

<script setup lang="ts">
import GIcon from "@components/icons/GIcon.vue"
import type { ProjectFilterType } from "../types"

defineProps<{
  modelValue: ProjectFilterType
  counts: Record<ProjectFilterType, number>
}>()

defineEmits<{ "update:modelValue": [value: ProjectFilterType] }>()

const items: Array<{ key: ProjectFilterType; label: string; icon?: string }> = [
  { key: "all", label: "全部" },
  { key: "running", label: "运行中", icon: "play" },
  { key: "stopped", label: "已停止", icon: "stop" },
  { key: "favorite", label: "收藏", icon: "star" },
  { key: "recent", label: "最近", icon: "clock" },
]
</script>
