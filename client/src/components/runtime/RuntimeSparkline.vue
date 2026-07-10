<template>
  <svg class="runtime-sparkline" viewBox="0 0 120 34" role="img" :aria-label="label">
    <polyline class="runtime-sparkline__line" :points="polyline" />
  </svg>
</template>

<script setup lang="ts">
import { computed } from 'vue'

const props = withDefaults(
  defineProps<{
    values: number[]
    label?: string
  }>(),
  {
    label: 'Runtime sparkline',
  },
)

const polyline = computed(() => {
  const values = props.values.map((value) => (Number.isFinite(value) ? Math.max(0, value) : 0))
  if (!values.length) return '0,28 120,28'

  const max = Math.max(...values, 1)
  const step = values.length <= 1 ? 120 : 120 / (values.length - 1)
  return values
    .map((value, index) => {
      const x = Math.round(index * step * 10) / 10
      const y = Math.round((30 - (value / max) * 26) * 10) / 10
      return `${x},${y}`
    })
    .join(' ')
})
</script>

<style scoped>
.runtime-sparkline {
  width: 100%;
  height: 34px;
  display: block;
  overflow: visible;
}

.runtime-sparkline__line {
  fill: none;
  stroke: currentColor;
  stroke-linecap: round;
  stroke-linejoin: round;
  stroke-width: 2.4;
  transition: points 180ms var(--ease-out);
}
</style>
