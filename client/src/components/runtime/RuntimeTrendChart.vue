<template>
  <section class="runtime-trend-chart">
    <header class="runtime-trend-chart__head">
      <div>
        <span>{{ eyebrow }}</span>
        <strong>{{ title }}</strong>
      </div>
      <slot name="meta" />
    </header>

    <svg viewBox="0 0 360 176" role="img" :aria-label="title">
      <g class="runtime-trend-chart__grid">
        <line
          v-for="tick in yAxisTicks"
          :key="`h-${tick.y}`"
          :x1="plotArea.left"
          :x2="plotArea.right"
          :y1="tick.y"
          :y2="tick.y" />
        <line
          v-for="x in xAxisTicks"
          :key="`v-${x.x}`"
          :x1="x.x"
          :x2="x.x"
          :y1="plotArea.top"
          :y2="plotArea.bottom" />
      </g>

      <g class="runtime-trend-chart__axis">
        <text
          v-for="tick in yAxisTicks"
          :key="`yl-${tick.y}`"
          class="runtime-trend-chart__y-label"
          :x="plotArea.left - 8"
          :y="tick.y">
          {{ tick.label }}
        </text>
        <text
          v-for="x in xAxisTicks"
          :key="`xl-${x.x}`"
          class="runtime-trend-chart__x-label"
          :x="x.x"
          :y="plotArea.bottom + 14">
          {{ x.label }}
        </text>
      </g>

      <polyline
        v-for="series in normalizedSeries"
        :key="series.name"
        class="runtime-trend-chart__line"
        :points="series.points"
        :style="{ color: series.color }" />
    </svg>

    <div v-if="normalizedSeries.length > 1" class="runtime-trend-chart__legend">
      <span v-for="series in normalizedSeries" :key="series.name">
        <i :style="{ background: series.color }" />
        {{ series.name }}
      </span>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed } from 'vue'

interface ChartSeries {
  name: string
  color: string
  values: number[]
}

const props = defineProps<{
  title: string
  eyebrow: string
  series: ChartSeries[]
  unit?: string
  timestamps?: number[]
}>()

const plotArea = {
  left: 40,
  right: 338,
  top: 24,
  bottom: 130,
}

const plotWidth = plotArea.right - plotArea.left
const plotHeight = plotArea.bottom - plotArea.top

const maxValue = computed(() =>
  Math.max(
    1,
    ...props.series.flatMap((series) =>
      series.values.map((value) => (Number.isFinite(value) ? Math.max(0, value) : 0)),
    ),
  ),
)

const normalizedSeries = computed(() => {
  const max = maxValue.value

  return props.series
    .map((series) => {
      const values = series.values.map((value) => (Number.isFinite(value) ? Math.max(0, value) : 0))
      if (!values.length) return { ...series, points: '' }

      const step = values.length <= 1 ? 0 : plotWidth / (values.length - 1)
      const points = values
        .map((value, index) => {
          const x = Math.round((plotArea.left + index * step) * 10) / 10
          const y = Math.round((plotArea.bottom - (value / max) * plotHeight) * 10) / 10
          return `${x},${y}`
        })
        .join(' ')

      return { ...series, points }
    })
    .filter((series) => series.points)
})

function formatAxisValue(value: number): string {
  if (props.unit === '%') return `${Math.round(value)}%`
  if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(1)}M`
  if (value >= 1_000) return `${(value / 1_000).toFixed(1)}K`
  return `${Math.round(value)}`
}

const yAxisTicks = computed(() => {
  const ticks = 5
  const labels = []
  for (let i = 0; i < ticks; i += 1) {
    const ratio = (ticks - 1 - i) / (ticks - 1)
    const value = maxValue.value * ratio
    const y = plotArea.top + i * (plotHeight / (ticks - 1))
    const suffix = props.unit && props.unit !== '%' ? ` ${props.unit}` : ''
    labels.push({
      y: Math.round(y * 10) / 10,
      label: `${formatAxisValue(value)}${suffix}`,
    })
  }
  return labels
})

function formatTime(timestamp: number): string {
  const date = new Date(timestamp)
  return `${String(date.getHours()).padStart(2, '0')}:${String(date.getMinutes()).padStart(2, '0')}`
}

const xAxisTicks = computed(() => {
  const timestamps = props.timestamps?.length
    ? props.timestamps.filter((t) => Number.isFinite(t))
    : []
  const count = props.series[0]?.values.length ?? 0

  if (count <= 1 || timestamps.length === 0) {
    return [
      { x: plotArea.left, label: '0' },
      { x: plotArea.right, label: count > 1 ? String(count) : '现在' },
    ]
  }

  const first = timestamps[0]
  const last = timestamps[timestamps.length - 1]
  const middle = timestamps[Math.floor((timestamps.length - 1) / 2)]

  return [
    { x: plotArea.left, label: formatTime(first) },
    { x: plotArea.left + plotWidth / 2, label: formatTime(middle) },
    { x: plotArea.right, label: formatTime(last) },
  ]
})
</script>

<style scoped>
.runtime-trend-chart {
  min-width: 0;
  display: grid;
  gap: var(--space-2);
  padding: var(--space-3);
  border: 1px solid var(--border-subtle);
  border-radius: 8px;
  background: var(--bg-input);
}

.runtime-trend-chart__head {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-3);
}

.runtime-trend-chart__head span,
.runtime-trend-chart__legend {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

.runtime-trend-chart__head strong {
  display: block;
  margin-top: 2px;
  color: var(--text-primary);
  font-size: var(--text-sm);
}

.runtime-trend-chart svg {
  width: 100%;
  height: 176px;
}

.runtime-trend-chart__grid line {
  stroke: var(--border-subtle);
  stroke-dasharray: 3 5;
}

.runtime-trend-chart__axis text {
  fill: var(--text-tertiary);
  font-size: 10px;
  dominant-baseline: middle;
}

.runtime-trend-chart__y-label {
  text-anchor: end;
}

.runtime-trend-chart__x-label {
  text-anchor: middle;
}

.runtime-trend-chart__line {
  fill: none;
  stroke: currentColor;
  stroke-linecap: round;
  stroke-linejoin: round;
  stroke-width: 2.4;
  transition: points 180ms var(--ease-out);
}

.runtime-trend-chart__legend {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-3);
}

.runtime-trend-chart__legend span {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
}

.runtime-trend-chart__legend i {
  width: 16px;
  height: 2px;
  border-radius: var(--radius-full);
}
</style>
