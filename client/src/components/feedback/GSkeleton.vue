<!--
  GSkeleton — 骨架屏
  ------------------------------------------------------------------
  用途：内容加载占位。支持 text / rect / circle 三种形态。
  Props:
    variant  text(行) | rect(矩形) | circle(圆)
    width    宽度（text 默认 100%）
    height   高度（text 默认字号行高）
    rows     text 模式行数
    rounded  自定义圆角
-->
<template>
  <div class="g-skeleton-wrap">
    <template v-if="variant === 'text'">
      <div
        v-for="i in rows"
        :key="i"
        class="g-skeleton g-skeleton--text"
        :style="{
          width: i === rows && rows > 1 ? '60%' : width,
          height: height ?? '12px',
          borderRadius: '2px',
        }" />
    </template>
    <div
      v-else
      class="g-skeleton"
      :class="`g-skeleton--${variant}`"
      :style="{ width, height, borderRadius: rounded }" />
  </div>
</template>

<script setup lang="ts">
withDefaults(
  defineProps<{
    variant?: 'text' | 'rect' | 'circle'
    width?: string
    height?: string
    rows?: number
    rounded?: string
  }>(),
  {
    variant: 'rect',
    width: '100%',
    rows: 1,
  },
)
</script>

<style scoped>
.g-skeleton-wrap {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}
.g-skeleton {
  display: block;
}
.g-skeleton--circle {
  border-radius: var(--radius-full) !important;
}
.g-skeleton--text {
  display: block;
}
</style>
