<template>
  <section class="overview-panel">
    <StatisticsCard
      label="运行中的隧道"
      :value="String(data.runningTunnel)"
      icon="router"
      :meta="`共 ${data.tunnelCount} 个`"
      tone="success" />
    <StatisticsCard
      label="当前连接"
      :value="String(data.currentConnection)"
      icon="plug-zap"
      :meta="`${data.averageRttMs.toFixed(0)} ms 平均 RTT`"
      tone="primary" />
    <StatisticsCard
      label="今日流量"
      :value="formatBytes(data.todayTraffic)"
      icon="arrow-right-left"
      :meta="`累计 ${formatBytes(data.totalTraffic)}`"
      tone="info" />
    <StatisticsCard
      label="健康评分"
      :value="String(Math.round(data.healthScore))"
      icon="shield-check"
      :meta="formatDuration(data.runtimeUptimeSeconds)"
      :tone="data.healthScore > 80 ? 'success' : 'warning'" />
  </section>
</template>

<script setup lang="ts">
import StatisticsCard from './StatisticsCard.vue'
import type { OverviewStatistics } from '@/monitoring/types'

defineProps<{
  data: OverviewStatistics
}>()

function formatBytes(value: number) {
  if (value >= 1024 ** 3) return `${(value / 1024 ** 3).toFixed(1)} GB`
  if (value >= 1024 ** 2) return `${(value / 1024 ** 2).toFixed(1)} MB`
  return `${(value / 1024).toFixed(0)} KB`
}

function formatDuration(seconds: number) {
  const hours = Math.floor(seconds / 3600)
  const minutes = Math.floor((seconds % 3600) / 60)
  return `${hours} 小时 ${minutes} 分钟运行`
}
</script>

<style scoped>
.overview-panel {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: var(--space-4);
}

@media (max-width: 1100px) {
  .overview-panel {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 620px) {
  .overview-panel {
    grid-template-columns: 1fr;
  }
}
</style>
