<template>
  <DashboardWidget
    title="Connection Statistics"
    icon="network"
  >
    <div class="connection-statistics">
      <div class="connection-statistics__numbers">
        <span>
          <strong>{{ connection.currentConnection }}</strong>
          Current
        </span>
        <span>
          <strong>{{ connection.success }}</strong>
          Success
        </span>
        <span>
          <strong>{{ connection.failure }}</strong>
          Failure
        </span>
        <span>
          <strong>{{ connection.reconnect }}</strong>
          Reconnect
        </span>
      </div>
      <svg
        class="connection-statistics__chart"
        viewBox="0 0 420 140"
        role="img"
        aria-label="Connection trend"
      >
        <path
          class="connection-statistics__grid"
          d="M20 24H400M20 68H400M20 112H400"
        />
        <polyline
          class="connection-statistics__line"
          :points="trendPoints"
        />
      </svg>
      <div class="connection-statistics__footer">
        <span>Total {{ connection.totalConnection }}</span>
        <span>RTT {{ connection.averageRttMs.toFixed(0) }} ms</span>
      </div>
    </div>
  </DashboardWidget>
</template>

<script setup lang="ts">
import { computed } from "vue"
import DashboardWidget from "./DashboardWidget.vue"
import type { ConnectionStatistics, ConnectionTrendPoint } from "@/monitoring/types"

const props = defineProps<{
  connection: ConnectionStatistics
  trend: ConnectionTrendPoint[]
}>()

const trendPoints = computed(() => {
  const maxValue = Math.max(1, ...props.trend.map((point) => point.current))
  const lastIndex = Math.max(1, props.trend.length - 1)
  return props.trend
    .map((point, index) => {
      const x = 20 + (index / lastIndex) * 380
      const y = 18 + 104 - (point.current / maxValue) * 104
      return `${x.toFixed(1)},${y.toFixed(1)}`
    })
    .join(" ")
})
</script>

<style scoped>
.connection-statistics {
  display: flex;
  min-height: 260px;
  flex-direction: column;
  gap: var(--space-3);
}

.connection-statistics__numbers {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: var(--space-2);
}

.connection-statistics__numbers span {
  min-width: 0;
  padding: var(--space-3);
  border: 1px solid var(--color-border-subtle);
  border-radius: var(--radius-md);
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

.connection-statistics__numbers strong {
  display: block;
  overflow: hidden;
  color: var(--text-primary);
  font-size: var(--text-lg);
  font-variant-numeric: tabular-nums;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.connection-statistics__chart {
  width: 100%;
  min-height: 140px;
}

.connection-statistics__grid {
  fill: none;
  stroke: var(--color-border-subtle);
}

.connection-statistics__line {
  fill: none;
  stroke: var(--color-warning);
  stroke-linecap: round;
  stroke-linejoin: round;
  stroke-width: 3;
}

.connection-statistics__footer {
  display: flex;
  justify-content: space-between;
  gap: var(--space-3);
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

@media (max-width: 640px) {
  .connection-statistics__numbers {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}
</style>
