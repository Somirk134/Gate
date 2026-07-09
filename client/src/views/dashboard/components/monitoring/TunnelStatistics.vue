<template>
  <DashboardWidget title="隧道列表" icon="router">
    <div class="tunnel-statistics">
      <div class="tunnel-statistics__summary">
        <span v-for="bucket in status" :key="bucket.label">
          <strong>{{ bucket.count }}</strong>
          {{ bucket.label }}
        </span>
      </div>
      <div class="tunnel-statistics__list">
        <article v-for="tunnel in tunnels" :key="tunnel.id" class="tunnel-statistics__item">
          <div>
            <strong>{{ tunnel.name }}</strong>
            <small>{{ tunnel.protocol.toUpperCase() }} · {{ tunnel.connections }} 个连接</small>
          </div>
          <span class="tunnel-statistics__status" :class="`is-${tunnel.status}`">
            {{ tunnel.status }}
          </span>
          <div class="tunnel-statistics__speed">
            <span>{{ formatSpeed(tunnel.uploadSpeedBps) }}</span>
            <span>{{ formatSpeed(tunnel.downloadSpeedBps) }}</span>
          </div>
          <div
            v-if="tunnel.protocol === 'http' || tunnel.protocol === 'https'"
            class="tunnel-statistics__http">
            <span>
              <strong>{{ formatCount(tunnel.requestCount ?? 0) }}</strong>
              Request
            </span>
            <span>
              <strong>{{ formatBytes(tunnel.trafficBytes ?? 0) }}</strong>
              Traffic
            </span>
            <span>
              <strong>{{ formatLatency(tunnel.averageResponseTimeMs ?? 0) }}</strong>
              Latency
            </span>
            <span>
              <strong>{{ formatPercent(1 - (tunnel.successRate ?? 0)) }}</strong>
              Error
            </span>
          </div>
        </article>
      </div>
    </div>
  </DashboardWidget>
</template>

<script setup lang="ts">
import DashboardWidget from './DashboardWidget.vue'
import type { DashboardTunnel } from '@/monitoring/types'

defineProps<{
  status: Array<{ label: string; count: number }>
  tunnels: DashboardTunnel[]
}>()

function formatSpeed(value: number) {
  if (value >= 1024 ** 2) return `${(value / 1024 ** 2).toFixed(1)} MB/s`
  return `${(value / 1024).toFixed(0)} KB/s`
}

function formatBytes(value: number) {
  if (value >= 1024 ** 3) return `${(value / 1024 ** 3).toFixed(1)} GB`
  if (value >= 1024 ** 2) return `${(value / 1024 ** 2).toFixed(1)} MB`
  if (value >= 1024) return `${(value / 1024).toFixed(0)} KB`
  return `${value} B`
}

function formatLatency(value: number) {
  if (value >= 1000) return `${(value / 1000).toFixed(1)} s`
  return `${value.toFixed(0)} ms`
}

function formatPercent(value: number) {
  return `${(Math.max(0, value) * 100).toFixed(1)}%`
}

function formatCount(value: number) {
  return new Intl.NumberFormat('zh-CN').format(value)
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

.tunnel-statistics__status.is-running {
  color: var(--color-success);
  background: var(--color-success-muted);
}
.tunnel-statistics__status.is-warning {
  color: var(--color-warning);
  background: var(--color-warning-muted);
}
.tunnel-statistics__status.is-stopped {
  color: var(--text-tertiary);
}

.tunnel-statistics__speed {
  display: flex;
  grid-column: 1 / -1;
  justify-content: space-between;
  gap: var(--space-3);
  color: var(--text-secondary);
  font-size: var(--text-xs);
  font-variant-numeric: tabular-nums;
}

.tunnel-statistics__http {
  display: grid;
  grid-column: 1 / -1;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: var(--space-2);
}

.tunnel-statistics__http span {
  min-width: 0;
  padding: var(--space-2);
  border: 1px solid var(--color-border-subtle);
  border-radius: var(--radius-sm);
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

.tunnel-statistics__http strong {
  display: block;
  overflow: hidden;
  color: var(--text-primary);
  font-size: var(--text-sm);
  font-variant-numeric: tabular-nums;
  text-overflow: ellipsis;
  white-space: nowrap;
}

@media (max-width: 640px) {
  .tunnel-statistics__http {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}
</style>
