<!--
  ProjectColorPicker — 项目颜色选择器
  ------------------------------------------------------------------
  网格色板，点击选中项目颜色。选中态带外边框。
-->
<template>
  <div class="project-color-grid">
    <button
      v-for="color in colors"
      :key="color.key"
      type="button"
      class="project-color-swatch"
      :class="{ 'project-color-swatch--active': modelValue === color.key }"
      :style="{ background: color.value + '22' }"
      :title="t(`project.colors.${color.key}`)"
      @click="$emit('update:modelValue', color.key)">
      <span class="project-color-swatch__dot" :style="{ background: color.value }" />
    </button>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { PROJECT_COLORS } from '../utils'
import type { ProjectColor } from '../types'

defineProps<{ modelValue: ProjectColor }>()
defineEmits<{ 'update:modelValue': [value: ProjectColor] }>()

const { t } = useI18n()
const colors = PROJECT_COLORS
</script>
