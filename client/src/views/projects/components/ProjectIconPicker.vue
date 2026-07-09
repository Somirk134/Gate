<!--
  ProjectIconPicker — 项目图标选择器
  ------------------------------------------------------------------
  网格展示所有可用 Lucide 图标，点击选中。
-->
<template>
  <div class="project-icon-grid">
    <button
      v-for="icon in icons"
      :key="icon.key"
      type="button"
      class="project-icon-cell"
      :class="{ 'project-icon-cell--active': modelValue === icon.key }"
      :title="icon.label"
      @click="$emit('update:modelValue', icon.key)"
    >
      <GIcon
        :name="icon.key"
        :size="16"
      />
    </button>
  </div>
</template>

<script setup lang="ts">
import GIcon from "@components/icons/GIcon.vue"
import { PROJECT_ICONS } from "../utils"

defineProps<{ modelValue: string }>()
defineEmits<{ "update:modelValue": [value: string] }>()

const icons = PROJECT_ICONS
</script>
