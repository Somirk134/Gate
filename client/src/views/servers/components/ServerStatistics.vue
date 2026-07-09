<!--
  ServerStatistics — 工作区 Statistics 标签
  ------------------------------------------------------------------
  展示服务器运行统计卡片组 + 流量占比进度环 + 详细统计表。
-->
<template>
  <div class="server-statistics">
    <div class="server-stat-grid">
      <div v-for="item in stats" :key="item.key" class="server-stat-card">
        <div
          class="server-stat-card__icon"
          :style="{ color: item.color, background: item.color + '1f' }">
          <GIcon :name="item.icon" :size="18" />
        </div>
        <div class="server-stat-card__body">
          <span class="server-stat-card__value">{{ item.value }}</span>
          <span class="server-stat-card__label">{{ item.label }}</span>
        </div>
      </div>
    </div>

    <!-- 进度环：流量占比 -->
    <div class="server-info-card" style="margin-top: var(--space-4)">
      <div class="server-info-card__title">
        <GIcon name="chart-pie" :size="12" />
        {{ t('server.statistics.trafficRatio') }}
      </div>
      <div class="server-statistics__rings">
        <div v-for="ring in rings" :key="ring.label" class="server-statistics__ring">
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
              class="server-statistics__ring-bar" />
          </svg>
          <div class="server-statistics__ring-text">
            <span class="server-statistics__ring-value">{{ ring.percent }}%</span>
            <span class="server-statistics__ring-label">{{ ring.label }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- 详细统计表 -->
    <div class="server-info-card" style="margin-top: var(--space-4)">
      <div class="server-info-card__title">
        <GIcon name="chart-bar" :size="12" />
        {{ t('server.statistics.detailTitle') }}
      </div>
      <div class="server-info-row">
        <span class="server-info-row__label">{{ t('server.metrics.uptime') }}</span>
        <span class="server-info-row__value mono">{{
          formatDuration(server.statistics.uptime, t)
        }}</span>
      </div>
      <div class="server-info-row">
        <span class="server-info-row__label">{{ t('server.metrics.tunnels') }}</span>
        <span class="server-info-row__value">{{ server.statistics.tunnelCount }}</span>
      </div>
      <div class="server-info-row">
        <span class="server-info-row__label">{{ t('server.metrics.projects') }}</span>
        <span class="server-info-row__value">{{ server.statistics.projectCount }}</span>
      </div>
      <div class="server-info-row">
        <span class="server-info-row__label">{{ t('server.metrics.activeConnections') }}</span>
        <span class="server-info-row__value">{{ server.monitor.connections.active }}</span>
      </div>
      <div class="server-info-row">
        <span class="server-info-row__label">{{ t('server.metrics.totalConnections') }}</span>
        <span class="server-info-row__value">{{
          formatNumber(server.monitor.connections.total)
        }}</span>
      </div>
      <div class="server-info-row">
        <span class="server-info-row__label">{{ t('tunnel.metrics.totalRequests') }}</span>
        <span class="server-info-row__value">{{ formatNumber(server.statistics.requests) }}</span>
      </div>
      <div class="server-info-row">
        <span class="server-info-row__label">{{ t('server.metrics.avgPing') }}</span>
        <span class="server-info-row__value">{{
          isOnline ? `${server.statistics.avgPing} ms` : '—'
        }}</span>
      </div>
      <div class="server-info-row">
        <span class="server-info-row__label">{{ t('tunnel.metrics.peakSpeed') }}</span>
        <span class="server-info-row__value mono">{{
          formatSpeed(server.statistics.peakSpeed)
        }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import GIcon from '@components/icons/GIcon.vue'
import type { Server } from '../types'
import { formatDuration, formatNumber, formatSpeed, isOnlineStatus } from '../utils'

const props = defineProps<{ server: Server }>()
const { t } = useI18n()

const ringStroke = 7
const ringRadius = (84 - ringStroke) / 2
const ringCircumference = 2 * Math.PI * ringRadius

const isOnline = computed(() => isOnlineStatus(props.server.status))

const stats = computed(() => [
  {
    key: 'uptime',
    label: t('server.metrics.uptime'),
    value: formatDuration(props.server.statistics.uptime, t),
    icon: 'clock',
    color: '#22C55E',
  },
  {
    key: 'tunnels',
    label: t('server.metrics.tunnels'),
    value: String(props.server.statistics.tunnelCount),
    icon: 'router',
    color: '#5B8DEF',
  },
  {
    key: 'projects',
    label: t('server.metrics.projects'),
    value: String(props.server.statistics.projectCount),
    icon: 'package',
    color: '#7C6FF2',
  },
  {
    key: 'activeConnections',
    label: t('server.metrics.activeConnections'),
    value: String(props.server.monitor.connections.active),
    icon: 'link',
    color: '#F59E0B',
  },
  {
    key: 'totalRequests',
    label: t('tunnel.metrics.totalRequests'),
    value: formatNumber(props.server.statistics.requests),
    icon: 'activity',
    color: '#06B6D4',
  },
  {
    key: 'avgPing',
    label: t('server.metrics.avgPing'),
    value: isOnline.value ? `${props.server.statistics.avgPing} ms` : '—',
    icon: 'gauge',
    color: '#EF4444',
  },
])

const rings = computed(() => {
  const total = props.server.traffic.totalUpload + props.server.traffic.totalDownload || 1
  const upPct = Math.round((props.server.traffic.totalUpload / total) * 100)
  const downPct = 100 - upPct
  const todayPct = Math.round(
    ((props.server.traffic.todayUpload + props.server.traffic.todayDownload) / total) * 100,
  )
  return [
    { label: t('tunnel.metrics.upload'), percent: upPct, color: '#22C55E' },
    { label: t('tunnel.metrics.download'), percent: downPct, color: '#5B8DEF' },
    { label: t('tunnel.today'), percent: Math.min(todayPct, 100), color: '#F59E0B' },
  ]
})
</script>
