<!--
  GCircleProgress — 环形进度
  ------------------------------------------------------------------
  用途：仪表盘/容量等环形指示。
  Props: value(0-100) / size(像素) / stroke / variant / showLabel
-->
<template>
  <div
    class="g-circle-progress"
    :style="{ width: `${size}px`, height: `${size}px` }"
  >
    <svg
      :width="size"
      :height="size"
      :viewBox="`0 0 ${size} ${size}`"
    >
      <circle
        class="g-circle-progress__track"
        :cx="size / 2"
        :cy="size / 2"
        :r="radius"
        :stroke-width="stroke"
        fill="none"
      />
      <circle
        class="g-circle-progress__bar"
        :class="`g-circle-progress__bar--${variant}`"
        :cx="size / 2"
        :cy="size / 2"
        :r="radius"
        :stroke-width="stroke"
        :stroke-dasharray="circumference"
        :stroke-dashoffset="dashoffset"
        stroke-linecap="round"
        fill="none"
        :transform="`rotate(-90 ${size / 2} ${size / 2})`"
      />
    </svg>
    <span
      v-if="showLabel"
      class="g-circle-progress__label"
    >
      {{ Math.round(clamp(value)) }}%
    </span>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue"

const props = withDefaults(
  defineProps<{
    value?: number
    size?: number
    stroke?: number
    variant?: "primary" | "success" | "warning" | "error"
    showLabel?: boolean
  }>(),
  {
    value: 0,
    size: 40,
    stroke: 4,
    variant: "primary",
    showLabel: true,
  },
)

const radius = computed(() => (props.size - props.stroke) / 2)
const circumference = computed(() => 2 * Math.PI * radius.value)
const dashoffset = computed(
  () => circumference.value * (1 - clamp(props.value) / 100),
)
function clamp(v: number) {
  return Math.max(0, Math.min(100, v))
}
</script>

<style scoped>
.g-circle-progress {
  position: relative;
  display: inline-flex;
  align-items: center;
  justify-content: center;
}
.g-circle-progress__track {
  stroke: var(--bg-surface-hover);
}
.g-circle-progress__bar {
  transition: stroke-dashoffset var(--duration-slow) var(--ease-out);
}
.g-circle-progress__bar--primary { stroke: var(--color-primary); }
.g-circle-progress__bar--success { stroke: var(--color-success); }
.g-circle-progress__bar--warning { stroke: var(--color-warning); }
.g-circle-progress__bar--error   { stroke: var(--color-error); }
.g-circle-progress__label {
  position: absolute;
  font-size: var(--text-xs);
  font-weight: var(--weight-medium);
  color: var(--text-primary);
  font-variant-numeric: tabular-nums;
}
</style>
