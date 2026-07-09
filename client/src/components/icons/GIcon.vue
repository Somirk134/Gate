<!--
  GIcon — 统一图标包装组件
  ------------------------------------------------------------------
  用途：项目内所有图标必须通过 GIcon 渲染，统一尺寸 / 颜色 / 状态。
  基于 Lucide Icons（见 icons/registry.ts）。

  用法：
    <GIcon name="dashboard" />
    <GIcon name="wifi" :size="20" color="var(--color-success)" />
    <GIcon name="settings" :size="16" spin />

  约定：
    - 默认尺寸 16px，颜色继承 currentColor
    - size 接受数字(px) 或预设 'xs'|'sm'|'md'|'lg'|'xl'
    - disabled 状态自动降低不透明度
-->
<template>
  <component
    :is="iconComp"
    class="g-icon"
    :class="[sizeClass, { 'g-icon--spin': spin, 'g-icon--disabled': disabled }]"
    :size="numericSize"
    :stroke-width="strokeWidth"
    :aria-hidden="decorative ? 'true' : undefined"
    :role="decorative ? undefined : 'img'" />
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { resolveIcon } from './registry'

const props = withDefaults(
  defineProps<{
    /** 图标名称，见 registry.ts */
    name: string
    /** 尺寸：预设档位或像素数字 */
    size?: number | 'xs' | 'sm' | 'md' | 'lg' | 'xl'
    /** 描边宽度，默认 1.75 */
    strokeWidth?: number
    /** 是否旋转动画（loading 等） */
    spin?: boolean
    /** 是否禁用 */
    disabled?: boolean
    /** 是否纯装饰（无障碍） */
    decorative?: boolean
  }>(),
  {
    size: 'md',
    strokeWidth: 1.75,
    spin: false,
    disabled: false,
    decorative: true,
  },
)

const iconComp = computed(() => resolveIcon(props.name))

const sizeMap: Record<string, number> = {
  xs: 12,
  sm: 14,
  md: 16,
  lg: 20,
  xl: 24,
}

const numericSize = computed(() =>
  typeof props.size === 'number' ? props.size : (sizeMap[props.size] ?? 16),
)

const sizeClass = computed(() => `g-icon--${props.size}`)
</script>

<style scoped>
.g-icon {
  display: inline-flex;
  flex-shrink: 0;
  color: currentColor;
  vertical-align: middle;
}

.g-icon--disabled {
  opacity: 0.4;
  cursor: not-allowed;
  pointer-events: none;
}

.g-icon--spin {
  animation: g-spin var(--duration-slow) var(--ease-linear) infinite;
}
</style>
