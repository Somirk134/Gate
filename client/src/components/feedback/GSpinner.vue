<!--
  GSpinner — 加载旋转图标
  ------------------------------------------------------------------
  用途：行内/按钮内加载指示。基于 Lucide loader 旋转。
  Props: size(sm|md|lg|数字) / variant(颜色)
-->
<template>
  <span class="g-spinner" :class="[`g-spinner--${variant}`]">
    <GIcon name="loader" :size="numericSize" spin />
  </span>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import GIcon from '@components/icons/GIcon.vue'

const props = withDefaults(
  defineProps<{
    size?: number | 'sm' | 'md' | 'lg'
    variant?: 'primary' | 'inherit' | 'neutral'
  }>(),
  {
    size: 'md',
    variant: 'primary',
  },
)

const sizeMap = { sm: 14, md: 18, lg: 24 }
const numericSize = computed(() =>
  typeof props.size === 'number' ? props.size : (sizeMap[props.size] ?? 18),
)
</script>

<style scoped>
.g-spinner {
  display: inline-flex;
  align-items: center;
  justify-content: center;
}
.g-spinner--primary {
  color: var(--color-primary);
}
.g-spinner--neutral {
  color: var(--text-tertiary);
}
.g-spinner--inherit {
  color: inherit;
}
</style>
