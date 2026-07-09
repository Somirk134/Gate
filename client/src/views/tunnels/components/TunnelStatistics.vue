<!--
  TunnelStatistics — 工作区 Statistics 标签
  ------------------------------------------------------------------
  展示隧道运行统计卡片组 + 协议分布饼图。
-->
<template>
  <div class="tunnel-statistics">
    <div class="tunnel-stat-grid">
      <div v-for="item in stats" :key="item.key" class="tunnel-stat-card">
        <div
          class="tunnel-stat-card__icon"
          :style="{ color: item.color, background: item.color + '1f' }">
          <GIcon :name="item.icon" :size="18" />
        </div>
        <div class="tunnel-stat-card__body">
          <span class="tunnel-stat-card__value">{{ item.value }}</span>
          <span class="tunnel-stat-card__label">{{ item.label }}</span>
        </div>
      </div>
    </div>

    <!-- 进度环：当前流量 vs 峰值 -->
    <div class="tunnel-info-card" style="margin-top: var(--space-4)">
      <div class="tunnel-info-card__title">
        <GIcon name="chart-pie" :size="12" />
        {{ t('tunnel.statistics.trafficRatio') }}
      </div>
      <div class="tunnel-statistics__rings">
        <div v-for="ring in rings" :key="ring.label" class="tunnel-statistics__ring">
          <svg width="84" height="84" viewBox="0 0 84 84">
            <circle
              cx="42"
              cy="42"
              :r="ringRadius"
              stroke="var(--bg-surface-hover)"
              :stroke-width="ringStroke"
              fill="none" />
            <circle
              cx="42"
              cy="42"
              :r="ringRadius"
              :stroke="ring.color"
              :stroke-width="ringStroke"
              :stroke-dasharray="ringCircumference"
              :stroke-dashoffset="ringCircumference * (1 - ring.percent / 100)"
              stroke-linecap="round"
              fill="none"
              transform="rotate(-90 42 42)"
              class="tunnel-statistics__ring-bar" />
          </svg>
          <div class="tunnel-statistics__ring-text">
            <span class="tunnel-statistics__ring-value">{{ ring.percent }}%</span>
            <span class="tunnel-statistics__ring-label">{{ ring.label }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 详细统计表 -->
    <div class="tunnel-info-card" style="margin-top: var(--space-4)">
      <div class="tunnel-info-card__title">
        <GIcon name="chart-bar" :size="12" />
        {{ t('tunnel.statistics.detailTitle') }}
      </div>
      <div class="tunnel-info-row">
        <span class="tunnel-info-row__label">{{ t('tunnel.metrics.uptime') }}</span>
        <span class="tunnel-info-row__value mono">{{
          formatDuration(tunnel.statistics.uptime, t)
        }}</span>
      </div>
      <div class="tunnel-info-row">
        <span class="tunnel-info-row__label">{{ t('tunnel.metrics.currentConnections') }}</span>
        <span class="tunnel-info-row__value">{{ tunnel.statistics.connections }}</span>
      </div>
      <div class="tunnel-info-row">
        <span class="tunnel-info-row__label">{{ t('tunnel.metrics.totalConnections') }}</span>
        <span class="tunnel-info-row__value">{{
          formatNumber(tunnel.statistics.totalConnections)
        }}</span>
      </div>
      <div class="tunnel-info-row">
        <span class="tunnel-info-row__label">{{ t('tunnel.metrics.totalRequests') }}</span>
        <span class="tunnel-info-row__value">{{ formatNumber(tunnel.statistics.requests) }}</span>
      </div>
      <div class="tunnel-info-row">
        <span class="tunnel-info-row__label">{{ t('tunnel.metrics.avgLatency') }}</span>
        <span class="tunnel-info-row__value">{{ tunnel.statistics.avgLatency }} ms</span>
      </div>
      <div class="tunnel-info-row">
        <span class="tunnel-info-row__label">{{ t('tunnel.metrics.peakSpeed') }}</span>
        <span class="tunnel-info-row__value mono">{{
          formatSpeed(tunnel.statistics.peakSpeed)
        }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import GIcon from '@components/icons/GIcon.vue'
import type { Tunnel } from '../types'
import { formatDuration, formatNumber, formatSpeed } from '../utils'

const props = defineProps<{ tunnel: Tunnel }>()
const { t } = useI18n()

const ringStroke = 7
const ringRadius = (84 - ringStroke) / 2
const ringCircumference = 2 * Math.PI * ringRadius

const stats = computed(() => [
  {
    key: 'uptime',
    label: t('tunnel.metrics.uptime'),
    value: formatDuration(props.tunnel.statistics.uptime, t),
    icon: 'clock',
    color: '#22C55E',
  },
  {
    key: 'currentConnections',
    label: t('tunnel.metrics.currentConnections'),
    value: String(props.tunnel.statistics.connections),
    icon: 'link',
    color: '#F59E0B',
  },
  {
    key: 'totalConnections',
    label: t('tunnel.metrics.totalConnections'),
    value: formatNumber(props.tunnel.statistics.totalConnections),
    icon: 'users',
    color: '#5B8DEF',
  },
  {
    key: 'totalRequests',
    label: t('tunnel.metrics.totalRequests'),
    value: formatNumber(props.tunnel.statistics.requests),
    icon: 'activity',
    color: '#7C6FF2',
  },
  {
    key: 'avgLatency',
    label: t('tunnel.metrics.avgLatency'),
    value: `${props.tunnel.statistics.avgLatency} ms`,
    icon: 'gauge',
    color: '#06B6D4',
  },
  {
    key: 'peakSpeed',
    label: t('tunnel.metrics.peakSpeed'),
    value: formatSpeed(props.tunnel.statistics.peakSpeed),
    icon: 'zap',
    color: '#EF4444',
  },
])

const rings = computed(() => {
  const total = props.tunnel.traffic.totalUpload + props.tunnel.traffic.totalDownload || 1
  const upPct = Math.round((props.tunnel.traffic.totalUpload / total) * 100)
  const downPct = 100 - upPct
  const todayPct = Math.round(
    ((props.tunnel.traffic.todayUpload + props.tunnel.traffic.todayDownload) / total) * 100,
  )
  return [
    { label: t('tunnel.metrics.upload'), percent: upPct, color: '#22C55E' },
    { label: t('tunnel.metrics.download'), percent: downPct, color: '#5B8DEF' },
    { label: t('tunnel.today'), percent: Math.min(todayPct, 100), color: '#F59E0B' },
  ]
})
</script>

<style scoped>
.tunnel-stat-card {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-4);
  background: var(--bg-card);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-card);
}

.tunnel-stat-card__icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  border-radius: var(--radius-md);
  flex-shrink: 0;
}

.tunnel-stat-card__body {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.tunnel-stat-card__value {
  font-size: var(--text-lg);
  font-weight: var(--weight-semibold);
  color: var(--text-primary);
  font-variant-numeric: tabular-nums;
  line-height: 1.2;
}

.tunnel-stat-card__label {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}

.tunnel-statistics__rings {
  display: flex;
  align-items: center;
  justify-content: space-around;
  gap: var(--space-4);
  flex-wrap: wrap;
  padding: var(--space-3) 0;
}

.tunnel-statistics__ring {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-2);
  position: relative;
}

.tunnel-statistics__ring-bar {
  transition: stroke-dashoffset var(--duration-slow) var(--ease-out);
}

.tunnel-statistics__ring-text {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1px;
}

.tunnel-statistics__ring-value {
  font-size: var(--text-md);
  font-weight: var(--weight-bold);
  color: var(--text-primary);
  font-variant-numeric: tabular-nums;
}

.tunnel-statistics__ring-label {
  font-size: 10px;
  color: var(--text-tertiary);
}
</style>
