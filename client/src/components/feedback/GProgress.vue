<!--
  GProgress — 线性进度条
  ------------------------------------------------------------------
  用途：上传/下载/任务进度。
  Props:
    value     0-100
    variant   primary | success | warning | error
    size      sm | md
    indeterminate  不确定状态（无具体值时滚动）
    showLabel 是否显示百分比文字
-->
<template>
  <div
    class="g-progress"
    :class="[`g-progress--${size}`]"
  >
    <div
      v-if="showLabel"
      class="g-progress__label"
    >
      <span>{{ label ?? `${Math.round(value)}%` }}</span>
    </div>
    <div class="g-progress__track">
      <div
        class="g-progress__bar"
        :class="[`g-progress__bar--${variant}`, { 'g-progress__bar--indeterminate': indeterminate }]"
        :style="{ width: indeterminate ? '40%' : `${clamp(value)}%` }"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
withDefaults(
  defineProps<{
    value?: number
    variant?: "primary" | "success" | "warning" | "error"
    size?: "sm" | "md"
    indeterminate?: boolean
    showLabel?: boolean
    label?: string
  }>(),
  {
    value: 0,
    variant: "primary",
    size: "md",
    indeterminate: false,
    showLabel: false,
  },
)

function clamp(v: number) {
  return Math.max(0, Math.min(100, v))
}
</script>

<style scoped>
.g-progress {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
  width: 100%;
}
.g-progress__label {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  font-variant-numeric: tabular-nums;
}
.g-progress__track {
  width: 100%;
  background: var(--bg-surface-hover);
  border-radius: var(--radius-full);
  overflow: hidden;
}
.g-progress--sm .g-progress__track { height: 3px; }
.g-progress--md .g-progress__track { height: 6px; }

.g-progress__bar {
  height: 100%;
  border-radius: var(--radius-full);
  transition: width var(--duration-slow) var(--ease-out);
}
.g-progress__bar--primary { background: var(--color-primary); }
.g-progress__bar--success { background: var(--color-success); }
.g-progress__bar--warning { background: var(--color-warning); }
.g-progress__bar--error   { background: var(--color-error); }

.g-progress__bar--indeterminate {
  animation: g-progress-indeterminate 1.4s var(--ease-in-out) infinite;
}
@keyframes g-progress-indeterminate {
  0%   { transform: translateX(-100%); }
  100% { transform: translateX(350%); }
}
</style>
