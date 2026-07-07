<template>
  <DashboardWidget title="Realtime Speed" icon="activity">
    <GEmptyState v-if="!points.length" title="暂无数据" description="暂无实时速度样本。真实流量产生后将在这里显示。" />
    <div v-else class="realtime-chart">
      <div class="realtime-chart__speed">
        <div>
          <span>Upload</span>
          <strong>{{ formatSpeed(latestUpload) }}</strong>
        </div>
        <div>
          <span>Download</span>
          <strong>{{ formatSpeed(latestDownload) }}</strong>
        </div>
      </div>
      <svg class="realtime-chart__svg" viewBox="0 0 420 180" role="img" aria-label="Realtime speed">
        <path class="realtime-chart__grid" d="M24 28H400M24 72H400M24 116H400M24 160H400" />
        <polyline class="realtime-chart__line realtime-chart__line--upload" :points="uploadPoints" />
        <polyline class="realtime-chart__line realtime-chart__line--download" :points="downloadPoints" />
      </svg>
    </div>
  </DashboardWidget>
</template>

<script setup lang="ts">
import { computed } from "vue"
import GEmptyState from "@components/feedback/GEmptyState.vue"
import DashboardWidget from "./DashboardWidget.vue"
import type { RealtimeSpeedPoint } from "@/monitoring/types"

const props = defineProps<{
  points: RealtimeSpeedPoint[]
}>()

const maxValue = computed(() =>
  Math.max(1, ...props.points.flatMap((point) => [point.uploadBps, point.downloadBps])),
)
const latest = computed(() => props.points[props.points.length - 1])
const latestUpload = computed(() => latest.value?.uploadBps ?? 0)
const latestDownload = computed(() => latest.value?.downloadBps ?? 0)
const uploadPoints = computed(() => linePoints("uploadBps"))
const downloadPoints = computed(() => linePoints("downloadBps"))

function linePoints(key: "uploadBps" | "downloadBps") {
  const width = 376
  const height = 140
  const lastIndex = Math.max(1, props.points.length - 1)
  return props.points
    .map((point, index) => {
      const x = 24 + (index / lastIndex) * width
      const y = 20 + height - (point[key] / maxValue.value) * height
      return `${x.toFixed(1)},${y.toFixed(1)}`
    })
    .join(" ")
}

function formatSpeed(value: number) {
  if (value >= 1024 ** 2) return `${(value / 1024 ** 2).toFixed(2)} MB/s`
  return `${(value / 1024).toFixed(0)} KB/s`
}
</script>

<style scoped>
.realtime-chart {
  display: flex;
  min-height: 260px;
  flex-direction: column;
  gap: var(--space-3);
}

.realtime-chart__speed {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--space-3);
}

.realtime-chart__speed div {
  min-width: 0;
  padding: var(--space-3);
  border: 1px solid var(--color-border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
}

.realtime-chart__speed span {
  display: block;
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

.realtime-chart__speed strong {
  display: block;
  overflow: hidden;
  color: var(--text-primary);
  font-size: var(--text-lg);
  font-weight: var(--weight-semibold);
  font-variant-numeric: tabular-nums;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.realtime-chart__svg {
  width: 100%;
  min-height: 180px;
}

.realtime-chart__grid {
  fill: none;
  stroke: var(--color-border-subtle);
  stroke-width: 1;
}

.realtime-chart__line {
  fill: none;
  stroke-linecap: round;
  stroke-linejoin: round;
  stroke-width: 3;
}

.realtime-chart__line--upload { stroke: var(--color-success); }
.realtime-chart__line--download { stroke: var(--color-info); }
</style>
