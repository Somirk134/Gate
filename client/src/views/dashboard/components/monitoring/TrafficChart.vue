<template>
  <DashboardWidget
    title="Traffic Trend"
    icon="chart-line"
  >
    <GEmptyState
      v-if="!points.length"
      title="暂无数据"
      description="暂无流量趋势样本。真实流量产生后将在这里显示。"
    />
    <div
      v-else
      class="traffic-chart"
    >
      <div class="traffic-chart__legend">
        <span><i class="traffic-chart__dot traffic-chart__dot--upload" />Upload</span>
        <span><i class="traffic-chart__dot traffic-chart__dot--download" />Download</span>
      </div>
      <svg
        class="traffic-chart__svg"
        viewBox="0 0 420 180"
        role="img"
        aria-label="Traffic trend"
      >
        <path
          class="traffic-chart__grid"
          d="M24 28H400M24 72H400M24 116H400M24 160H400"
        />
        <polyline
          class="traffic-chart__line traffic-chart__line--upload"
          :points="uploadPoints"
        />
        <polyline
          class="traffic-chart__line traffic-chart__line--download"
          :points="downloadPoints"
        />
      </svg>
      <div class="traffic-chart__summary">
        <span>Today {{ formatBytes(todayUpload + todayDownload) }}</span>
        <span>Peak {{ formatBytes(peakBytes) }}/h</span>
      </div>
    </div>
  </DashboardWidget>
</template>

<script setup lang="ts">
import { computed } from "vue"
import GEmptyState from "@components/feedback/GEmptyState.vue"
import DashboardWidget from "./DashboardWidget.vue"
import type { TrafficTrendPoint } from "@/monitoring/types"

const props = defineProps<{
  points: TrafficTrendPoint[]
}>()

const chart = {
  width: 420,
  height: 180,
  left: 24,
  right: 20,
  top: 20,
  bottom: 20,
}

const maxValue = computed(() =>
  Math.max(1, ...props.points.flatMap((point) => [point.uploadBytes, point.downloadBytes])),
)

const todayUpload = computed(() =>
  props.points.reduce((total, point) => total + point.uploadBytes, 0),
)

const todayDownload = computed(() =>
  props.points.reduce((total, point) => total + point.downloadBytes, 0),
)

const peakBytes = computed(() =>
  Math.max(0, ...props.points.flatMap((point) => [point.uploadBytes, point.downloadBytes])),
)

const uploadPoints = computed(() => linePoints("uploadBytes"))
const downloadPoints = computed(() => linePoints("downloadBytes"))

function linePoints(key: "uploadBytes" | "downloadBytes") {
  const usableWidth = chart.width - chart.left - chart.right
  const usableHeight = chart.height - chart.top - chart.bottom
  const lastIndex = Math.max(1, props.points.length - 1)
  return props.points
    .map((point, index) => {
      const x = chart.left + (index / lastIndex) * usableWidth
      const y = chart.top + usableHeight - (point[key] / maxValue.value) * usableHeight
      return `${x.toFixed(1)},${y.toFixed(1)}`
    })
    .join(" ")
}

function formatBytes(value: number) {
  if (value >= 1024 ** 3) return `${(value / 1024 ** 3).toFixed(1)} GB`
  if (value >= 1024 ** 2) return `${(value / 1024 ** 2).toFixed(1)} MB`
  return `${(value / 1024).toFixed(0)} KB`
}
</script>

<style scoped>
.traffic-chart {
  display: flex;
  min-height: 260px;
  flex-direction: column;
  gap: var(--space-3);
}

.traffic-chart__legend,
.traffic-chart__summary {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

.traffic-chart__legend span {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
}

.traffic-chart__dot {
  width: 8px;
  height: 8px;
  border-radius: var(--radius-full);
}

.traffic-chart__dot--upload { background: var(--color-success); }
.traffic-chart__dot--download { background: var(--color-primary); }

.traffic-chart__svg {
  width: 100%;
  min-height: 180px;
}

.traffic-chart__grid {
  fill: none;
  stroke: var(--color-border-subtle);
  stroke-width: 1;
}

.traffic-chart__line {
  fill: none;
  stroke-linecap: round;
  stroke-linejoin: round;
  stroke-width: 3;
}

.traffic-chart__line--upload { stroke: var(--color-success); }
.traffic-chart__line--download { stroke: var(--color-primary); }

@media (max-width: 640px) {
  .traffic-chart__summary {
    align-items: flex-start;
    flex-direction: column;
  }
}
</style>
