<!--
  ServerStatistics — 工作区 Statistics 标签
  ------------------------------------------------------------------
  展示服务器运行统计卡片组 + 流量占比进度环 + 详细统计表。
-->
<template>
  <div class="server-statistics">
    <div class="server-stat-grid">
      <div v-for="item in stats" :key="item.label" class="server-stat-card">
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
        流量占比
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
        详细统计
      </div>
      <div class="server-info-row">
        <span class="server-info-row__label">运行时长</span>
        <span class="server-info-row__value mono">{{
          formatDuration(server.statistics.uptime)
        }}</span>
      </div>
      <div class="server-info-row">
        <span class="server-info-row__label">隧道数</span>
        <span class="server-info-row__value">{{ server.statistics.tunnelCount }}</span>
      </div>
      <div class="server-info-row">
        <span class="server-info-row__label">项目数</span>
        <span class="server-info-row__value">{{ server.statistics.projectCount }}</span>
      </div>
      <div class="server-info-row">
        <span class="server-info-row__label">活动连接</span>
        <span class="server-info-row__value">{{ server.monitor.connections.active }}</span>
      </div>
      <div class="server-info-row">
        <span class="server-info-row__label">累计连接</span>
        <span class="server-info-row__value">{{
          formatNumber(server.monitor.connections.total)
        }}</span>
      </div>
      <div class="server-info-row">
        <span class="server-info-row__label">累计请求</span>
        <span class="server-info-row__value">{{ formatNumber(server.statistics.requests) }}</span>
      </div>
      <div class="server-info-row">
        <span class="server-info-row__label">平均 Ping</span>
        <span class="server-info-row__value">{{
          isOnline ? `${server.statistics.avgPing} ms` : '—'
        }}</span>
      </div>
      <div class="server-info-row">
        <span class="server-info-row__label">峰值速度</span>
        <span class="server-info-row__value mono">{{
          formatSpeed(server.statistics.peakSpeed)
        }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import GIcon from '@components/icons/GIcon.vue'
import type { Server } from '../types'
import { formatDuration, formatNumber, formatSpeed, isOnlineStatus } from '../utils'

const props = defineProps<{ server: Server }>()

const ringStroke = 7
const ringRadius = (84 - ringStroke) / 2
const ringCircumference = 2 * Math.PI * ringRadius

const isOnline = computed(() => isOnlineStatus(props.server.status))

const stats = computed(() => [
  {
    label: '运行时长',
    value: formatDuration(props.server.statistics.uptime),
    icon: 'clock',
    color: '#22C55E',
  },
  {
    label: '隧道数',
    value: String(props.server.statistics.tunnelCount),
    icon: 'router',
    color: '#5B8DEF',
  },
  {
    label: '项目数',
    value: String(props.server.statistics.projectCount),
    icon: 'package',
    color: '#7C6FF2',
  },
  {
    label: '活动连接',
    value: String(props.server.monitor.connections.active),
    icon: 'link',
    color: '#F59E0B',
  },
  {
    label: '累计请求',
    value: formatNumber(props.server.statistics.requests),
    icon: 'activity',
    color: '#06B6D4',
  },
  {
    label: '平均 Ping',
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
    { label: '上传', percent: upPct, color: '#22C55E' },
    { label: '下载', percent: downPct, color: '#5B8DEF' },
    { label: '今日', percent: Math.min(todayPct, 100), color: '#F59E0B' },
  ]
})
</script>
