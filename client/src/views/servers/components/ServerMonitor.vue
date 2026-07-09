<!--
  ServerMonitor — 工作区 Monitor 标签
  ------------------------------------------------------------------
  实时监控面板：CPU / 内存 / 磁盘 / 负载 / 网络 / 流量 / 连接。
  采用 Card / Mini Chart / Progress / Circle，不使用复杂图表。
  数据来自 Runtime Store；服务端未上报时保持空值。
-->
<template>
  <div class="server-monitor">
    <!-- 顶部实时指标 -->
    <div class="server-stat-grid">
      <div class="server-stat-card">
        <div class="server-stat-card__icon" style="color: #5b8def; background: #5b8def1f">
          <GIcon name="cpu" :size="18" />
        </div>
        <div class="server-stat-card__body">
          <span class="server-stat-card__value"
            >{{ server.monitor.cpu.percent }}<span class="server-monitor__unit">%</span></span
          >
          <span class="server-stat-card__label">{{ t('server.monitor.cpuUsage') }}</span>
        </div>
      </div>
      <div class="server-stat-card">
        <div class="server-stat-card__icon" style="color: #7c6ff2; background: #7c6ff21f">
          <GIcon name="memory-stick" :size="18" />
        </div>
        <div class="server-stat-card__body">
          <span class="server-stat-card__value"
            >{{ server.monitor.memory.percent }}<span class="server-monitor__unit">%</span></span
          >
          <span class="server-stat-card__label">{{ t('server.monitor.memoryUsage') }}</span>
        </div>
      </div>
      <div class="server-stat-card">
        <div class="server-stat-card__icon" style="color: #f59e0b; background: #f59e0b1f">
          <GIcon name="hard-drive" :size="18" />
        </div>
        <div class="server-stat-card__body">
          <span class="server-stat-card__value"
            >{{ server.monitor.disk.percent }}<span class="server-monitor__unit">%</span></span
          >
          <span class="server-stat-card__label">{{ t('server.monitor.diskUsage') }}</span>
        </div>
      </div>
      <div class="server-stat-card">
        <div class="server-stat-card__icon" style="color: #06b6d4; background: #06b6d41f">
          <GIcon name="link" :size="18" />
        </div>
        <div class="server-stat-card__body">
          <span class="server-stat-card__value">{{ server.monitor.connections.active }}</span>
          <span class="server-stat-card__label">{{ t('server.metrics.activeConnections') }}</span>
        </div>
      </div>
    </div>

    <!-- 实时速度曲线 -->
    <div class="server-info-card" style="margin-top: var(--space-4)">
      <div class="server-info-card__title">
        <GIcon name="chart-line" :size="12" />
        {{ t('server.monitor.networkSpeedTitle') }}
        <span class="server-monitor__live">
          <span class="server-monitor__live-dot" />
          LIVE
        </span>
      </div>
      <div class="server-monitor__chart-wrap">
        <svg class="server-traffic__chart" viewBox="0 0 600 180" preserveAspectRatio="none">
          <defs>
            <linearGradient id="server-monitor-up" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stop-color="#22C55E" stop-opacity="0.30" />
              <stop offset="100%" stop-color="#22C55E" stop-opacity="0" />
            </linearGradient>
            <linearGradient id="server-monitor-down" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stop-color="#5B8DEF" stop-opacity="0.30" />
              <stop offset="100%" stop-color="#5B8DEF" stop-opacity="0" />
            </linearGradient>
          </defs>
          <line
            v-for="i in 3"
            :key="`mg-${i}`"
            x1="0"
            :x2="600"
            :y1="i * 45"
            :y2="i * 45"
            stroke="var(--color-border-subtle)"
            stroke-width="1" />
          <path :d="downArea" fill="url(#server-monitor-down)" />
          <path :d="downPath" fill="none" stroke="#5B8DEF" stroke-width="2" />
          <path :d="upArea" fill="url(#server-monitor-up)" />
          <path :d="upPath" fill="none" stroke="#22C55E" stroke-width="2" />
        </svg>
        <div class="server-traffic__legend">
          <span class="server-traffic__legend-item">
            <span class="server-traffic__legend-dot" style="background: #5b8def" />
            {{ t('server.monitor.downloadValue', { value: formatSpeed(server.traffic.downloadSpeed) }) }}
          </span>
          <span class="server-traffic__legend-item">
            <span class="server-traffic__legend-dot" style="background: #22c55e" />
            {{ t('server.monitor.uploadValue', { value: formatSpeed(server.traffic.uploadSpeed) }) }}
          </span>
        </div>
      </div>
    </div>

    <!-- 资源占用条 -->
    <div class="server-info-card" style="margin-top: var(--space-4)">
      <div class="server-info-card__title">
        <GIcon name="activity" :size="12" />
        {{ t('server.inspector.resourceUsage') }}
      </div>
      <div class="server-monitor__resource">
        <div v-for="r in resources" :key="r.label" class="server-monitor__resource-item">
          <div class="server-monitor__resource-head">
            <GIcon :name="r.icon" :size="13" />
            <span class="server-monitor__resource-label">{{ r.label }}</span>
            <span class="server-monitor__resource-value">{{ r.value }}</span>
          </div>
          <div class="server-monitor__bar">
            <div
              class="server-monitor__bar-fill"
              :style="{ width: `${r.percent}%`, background: r.color }" />
          </div>
        </div>
      </div>
    </div>

    <!-- 负载 -->
    <div class="server-info-card" style="margin-top: var(--space-4)">
      <div class="server-info-card__title">
        <GIcon name="gauge" :size="12" />
        {{ t('server.monitor.systemLoad') }}
      </div>
      <div class="server-info-row">
        <span class="server-info-row__label">{{ t('server.monitor.load1') }}</span>
        <span class="server-info-row__value mono">{{ server.monitor.load.load1 }}</span>
      </div>
      <div class="server-info-row">
        <span class="server-info-row__label">{{ t('server.monitor.load5') }}</span>
        <span class="server-info-row__value mono">{{ server.monitor.load.load5 }}</span>
      </div>
      <div class="server-info-row">
        <span class="server-info-row__label">{{ t('server.monitor.load15') }}</span>
        <span class="server-info-row__value mono">{{ server.monitor.load.load15 }}</span>
      </div>
      <div class="server-info-row">
        <span class="server-info-row__label">{{ t('server.monitor.cpuCores') }}</span>
        <span class="server-info-row__value mono">{{ server.monitor.load.cores }}</span>
      </div>
    </div>

    <p class="server-connection__hint">
      <GIcon name="info-circle" :size="12" />
      {{ t('server.monitor.hint') }}
    </p>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import GIcon from '@components/icons/GIcon.vue'
import type { Server } from '../types'
import { formatSpeed } from '../utils'

const props = defineProps<{ server: Server }>()
const { t } = useI18n()

function buildPath(key: 'upload' | 'download'): string {
  const h = props.server.traffic.history
  if (!h.length) return ''
  const max = Math.max(...h.map((p) => p[key]), 1)
  const stepX = h.length > 1 ? 600 / (h.length - 1) : 0
  return h
    .map((p, i) => {
      const x = i * stepX
      const y = 175 - (p[key] / max) * 160
      return `${i === 0 ? 'M' : 'L'} ${x.toFixed(1)} ${y.toFixed(1)}`
    })
    .join(' ')
}

function buildArea(key: 'upload' | 'download'): string {
  const path = buildPath(key)
  if (!path) return ''
  return `${path} L 600 180 L 0 180 Z`
}

const upPath = computed(() => buildPath('upload'))
const upArea = computed(() => buildArea('upload'))
const downPath = computed(() => buildPath('download'))
const downArea = computed(() => buildArea('download'))

const resources = computed(() => [
  {
    label: 'CPU',
    icon: 'cpu',
    value: `${props.server.monitor.cpu.used}/${props.server.monitor.cpu.total} ${props.server.monitor.cpu.unit}`,
    percent: props.server.monitor.cpu.percent,
    color: 'var(--color-primary)',
  },
  {
    label: t('common.memory'),
    icon: 'memory-stick',
    value: `${props.server.monitor.memory.used}/${props.server.monitor.memory.total} ${props.server.monitor.memory.unit}`,
    percent: props.server.monitor.memory.percent,
    color: 'var(--color-secondary)',
  },
  {
    label: t('server.disk'),
    icon: 'hard-drive',
    value: `${props.server.monitor.disk.used}/${props.server.monitor.disk.total} ${props.server.monitor.disk.unit}`,
    percent: props.server.monitor.disk.percent,
    color: 'var(--color-warning)',
  },
])
</script>
