<template>
  <DashboardWidget title="Runtime Status" icon="circuit-board">
    <div class="runtime-panel">
      <div v-for="item in items" :key="item.label" class="runtime-panel__row">
        <span>{{ item.label }}</span>
        <strong>{{ item.value }}</strong>
        <div class="runtime-panel__bar" :aria-label="item.label">
          <i :style="{ width: `${item.percent}%` }" />
        </div>
      </div>
    </div>
  </DashboardWidget>
</template>

<script setup lang="ts">
import { computed } from "vue"
import DashboardWidget from "./DashboardWidget.vue"
import type { RuntimeStatistics } from "@/monitoring/types"

const props = defineProps<{
  runtime: RuntimeStatistics
}>()

const items = computed(() => [
  { label: "Running Task", value: props.runtime.runningTask, percent: Math.min(100, props.runtime.runningTask) },
  { label: "Worker Count", value: props.runtime.workerCount, percent: Math.min(100, props.runtime.workerCount * 8) },
  { label: "Scheduler Queue", value: props.runtime.schedulerQueue, percent: Math.min(100, props.runtime.schedulerQueue * 2) },
  { label: "Buffer Usage", value: `${props.runtime.bufferUsage.toFixed(0)}%`, percent: props.runtime.bufferUsage },
  { label: "Session Count", value: props.runtime.sessionCount, percent: Math.min(100, props.runtime.sessionCount * 1.6) },
])
</script>

<style scoped>
.runtime-panel {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.runtime-panel__row {
  display: grid;
  grid-template-columns: minmax(100px, 1fr) auto;
  align-items: center;
  gap: var(--space-2) var(--space-3);
}

.runtime-panel__row span {
  overflow: hidden;
  color: var(--text-secondary);
  font-size: var(--text-sm);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.runtime-panel__row strong {
  color: var(--text-primary);
  font-size: var(--text-sm);
  font-variant-numeric: tabular-nums;
}

.runtime-panel__bar {
  grid-column: 1 / -1;
  height: 6px;
  overflow: hidden;
  border-radius: var(--radius-full);
  background: var(--bg-surface-hover);
}

.runtime-panel__bar i {
  display: block;
  height: 100%;
  border-radius: inherit;
  background: linear-gradient(90deg, var(--color-primary), var(--color-success));
}
</style>
