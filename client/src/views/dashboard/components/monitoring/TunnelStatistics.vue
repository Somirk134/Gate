<template>
  <DashboardWidget
    title="Tunnel List"
    icon="router"
  >
    <div class="tunnel-statistics">
      <div class="tunnel-statistics__summary">
        <span
          v-for="bucket in status"
          :key="bucket.label"
        >
          <strong>{{ bucket.count }}</strong>
          {{ bucket.label }}
        </span>
      </div>
      <div class="tunnel-statistics__list">
        <article
          v-for="tunnel in tunnels"
          :key="tunnel.id"
          class="tunnel-statistics__item"
        >
          <div>
            <strong>{{ tunnel.name }}</strong>
            <small>{{ tunnel.protocol.toUpperCase() }} · {{ tunnel.connections }} conn</small>
          </div>
          <span
            class="tunnel-statistics__status"
            :class="`is-${tunnel.status}`"
          >
            {{ tunnel.status }}
          </span>
          <div class="tunnel-statistics__speed">
            <span>{{ formatSpeed(tunnel.uploadSpeedBps) }}</span>
            <span>{{ formatSpeed(tunnel.downloadSpeedBps) }}</span>
          </div>
        </article>
      </div>
    </div>
  </DashboardWidget>
</template>

<script setup lang="ts">
import DashboardWidget from "./DashboardWidget.vue"
import type { DashboardTunnel } from "@/monitoring/types"

defineProps<{
  status: Array<{ label: string; count: number }>
  tunnels: DashboardTunnel[]
}>()

function formatSpeed(value: number) {
  if (value >= 1024 ** 2) return `${(value / 1024 ** 2).toFixed(1)} MB/s`
  return `${(value / 1024).toFixed(0)} KB/s`
}
</script>

<style scoped>
.tunnel-statistics {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.tunnel-statistics__summary {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: var(--space-2);
}

.tunnel-statistics__summary span {
  min-width: 0;
  padding: var(--space-3);
  border: 1px solid var(--color-border-subtle);
  border-radius: var(--radius-md);
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

.tunnel-statistics__summary strong {
  display: block;
  color: var(--text-primary);
  font-size: var(--text-lg);
  font-variant-numeric: tabular-nums;
}

.tunnel-statistics__list {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.tunnel-statistics__item {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  gap: var(--space-2) var(--space-3);
  padding: var(--space-3);
  border: 1px solid var(--color-border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
}

.tunnel-statistics__item strong,
.tunnel-statistics__item small {
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.tunnel-statistics__item strong {
  color: var(--text-primary);
  font-size: var(--text-sm);
}

.tunnel-statistics__item small {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

.tunnel-statistics__status {
  align-self: start;
  padding: 2px 8px;
  border-radius: var(--radius-full);
  color: var(--text-tertiary);
  background: var(--bg-surface-hover);
  font-size: var(--text-xs);
}

.tunnel-statistics__status.is-running { color: var(--color-success); background: var(--color-success-muted); }
.tunnel-statistics__status.is-warning { color: var(--color-warning); background: var(--color-warning-muted); }
.tunnel-statistics__status.is-stopped { color: var(--text-tertiary); }

.tunnel-statistics__speed {
  display: flex;
  grid-column: 1 / -1;
  justify-content: space-between;
  gap: var(--space-3);
  color: var(--text-secondary);
  font-size: var(--text-xs);
  font-variant-numeric: tabular-nums;
}
</style>
