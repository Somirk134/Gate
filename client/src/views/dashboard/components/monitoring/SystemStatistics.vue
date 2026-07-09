<template>
  <DashboardWidget title="系统统计" icon="cpu">
    <div class="system-statistics">
      <div v-for="item in items" :key="item.label" class="system-statistics__meter">
        <div class="system-statistics__meter-head">
          <span>{{ item.label }}</span>
          <strong>{{ item.value }}</strong>
        </div>
        <div class="system-statistics__bar">
          <i :style="{ width: `${item.percent}%` }" />
        </div>
      </div>
      <div class="system-statistics__details">
        <span>线程 {{ system.threadCount }}</span>
        <span>打开文件 {{ system.openFile ?? '预留' }}</span>
        <span>运行时间 {{ formatDuration(system.processUptimeSeconds) }}</span>
      </div>
    </div>
  </DashboardWidget>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import DashboardWidget from './DashboardWidget.vue'
import type { SystemStatistics } from '@/monitoring/types'

const props = defineProps<{
  system: SystemStatistics
}>()

const items = computed(() => [
  {
    label: 'CPU 使用率',
    value: `${props.system.cpuUsage.toFixed(0)}%`,
    percent: props.system.cpuUsage,
  },
  {
    label: '内存使用率',
    value: `${props.system.memoryUsage.toFixed(0)}%`,
    percent: props.system.memoryUsage,
  },
  {
    label: '磁盘使用率',
    value: props.system.diskUsage ? `${props.system.diskUsage.toFixed(0)}%` : '预留',
    percent: props.system.diskUsage ?? 0,
  },
])

function formatDuration(seconds: number) {
  const hours = Math.floor(seconds / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  return `${hours} 小时 ${minutes} 分钟`
}
</script>

<style scoped>
.system-statistics {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.system-statistics__meter {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.system-statistics__meter-head,
.system-statistics__details {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  color: var(--text-secondary);
  font-size: var(--text-sm);
}

.system-statistics__meter-head strong {
  color: var(--text-primary);
  font-variant-numeric: tabular-nums;
}

.system-statistics__bar {
  height: 8px;
  overflow: hidden;
  border-radius: var(--radius-full);
  background: var(--bg-surface-hover);
}

.system-statistics__bar i {
  display: block;
  height: 100%;
  border-radius: inherit;
  background: linear-gradient(90deg, var(--color-info), var(--color-warning));
}

.system-statistics__details {
  align-items: flex-start;
  flex-direction: column;
  padding-top: var(--space-2);
  border-top: 1px solid var(--color-border-subtle);
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}
</style>
