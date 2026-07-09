<!--
  ServerInspector — 右侧实时检查器
  ------------------------------------------------------------------
  实时展示选中服务器：基础信息 / 实时统计 / 资源 / 流量趋势 / 属性。
-->
<template>
  <div class="server-inspector" :style="colorVars">
    <header class="server-inspector__header">
      <GIcon name="activity" :size="14" />
      <span>{{ t('server.inspector.title') }}</span>
      <span class="server-inspector__live">
        <span class="server-inspector__live-dot" />
        LIVE
      </span>
    </header>

    <div class="server-inspector__body">
      <!-- Hero -->
      <div class="server-inspector__hero">
        <span class="server-inspector__hero-icon">
          <GIcon :name="kindPreset.icon" :size="24" />
        </span>
        <div class="server-inspector__hero-text">
          <span class="server-inspector__hero-name" :title="server.name">{{ server.name }}</span>
          <ServerStatus :status="server.status" size="sm" />
        </div>
      </div>

      <!-- 实时统计 -->
      <div class="server-inspector__group">
        <div class="server-inspector__group-title">{{ t('server.inspector.realtimeStats') }}</div>
        <div class="server-inspector__stat">
          <div class="server-inspector__stat-item">
            <GIcon name="arrow-up" :size="12" />
            <span class="server-inspector__stat-label">{{ t('tunnel.metrics.upload') }}</span>
            <span class="server-inspector__stat-value">{{
              formatSpeed(server.traffic.uploadSpeed)
            }}</span>
          </div>
          <div class="server-inspector__stat-item">
            <GIcon name="arrow-down" :size="12" />
            <span class="server-inspector__stat-label">{{ t('tunnel.metrics.download') }}</span>
            <span class="server-inspector__stat-value">{{
              formatSpeed(server.traffic.downloadSpeed)
            }}</span>
          </div>
          <div class="server-inspector__stat-item">
            <GIcon name="cpu" :size="12" />
            <span class="server-inspector__stat-label">CPU</span>
            <span class="server-inspector__stat-value">{{ server.monitor.cpu.percent }}%</span>
          </div>
          <div class="server-inspector__stat-item">
            <GIcon name="memory-stick" :size="12" />
            <span class="server-inspector__stat-label">{{ t('common.memory') }}</span>
            <span class="server-inspector__stat-value">{{ server.monitor.memory.percent }}%</span>
          </div>
        </div>
      </div>

      <!-- Mini Chart -->
      <div class="server-inspector__group">
        <div class="server-inspector__group-title">{{ t('server.inspector.trafficTrend') }}</div>
        <svg class="server-mini-chart" viewBox="0 0 200 80" preserveAspectRatio="none">
          <defs>
            <linearGradient :id="`grad-${server.id}`" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" :stop-color="kindPreset.color" stop-opacity="0.35" />
              <stop offset="100%" :stop-color="kindPreset.color" stop-opacity="0" />
            </linearGradient>
          </defs>
          <path :d="downloadPath" fill="none" :stroke="kindPreset.color" stroke-width="1.5" />
          <path :d="downloadArea" :fill="`url(#grad-${server.id})`" />
        </svg>
      </div>

      <!-- 基础信息 -->
      <div class="server-inspector__group">
        <div class="server-inspector__group-title">{{ t('server.inspector.basic') }}</div>
        <div class="server-inspector__row">
          <span class="server-inspector__label">{{ t('server.overview.type') }}</span>
          <ServerBadge :kind="server.kind" size="sm" />
        </div>
        <div class="server-inspector__row">
          <span class="server-inspector__label">{{ t('server.overview.publicIp') }}</span>
          <span class="server-inspector__value mono">{{ server.publicIp }}</span>
        </div>
        <div class="server-inspector__row">
          <span class="server-inspector__label">{{ t('server.overview.region') }}</span>
          <span class="server-inspector__value">{{ server.region }}</span>
        </div>
        <div class="server-inspector__row">
          <span class="server-inspector__label">{{ t('server.overview.version') }}</span>
          <span class="server-inspector__value mono">{{ server.version }}</span>
        </div>
        <div class="server-inspector__row">
          <span class="server-inspector__label">{{ t('server.overview.connectionMethod') }}</span>
          <span class="server-inspector__value mono">{{
            server.connectionMethod.toUpperCase()
          }}</span>
        </div>
        <div class="server-inspector__row">
          <span class="server-inspector__label">{{ t('server.detail.autoConnect') }}</span>
          <span class="server-inspector__value">
            <GIcon
              :name="server.settings.autoConnect ? 'check' : 'close'"
              :size="12"
              :class="server.settings.autoConnect ? 'on' : 'off'" />
            {{ server.settings.autoConnect ? t('common.enabled') : t('common.disabled') }}
          </span>
        </div>
      </div>

      <!-- 资源 -->
      <div class="server-inspector__group">
        <div class="server-inspector__group-title">{{ t('server.inspector.resourceUsage') }}</div>
        <div class="server-inspector__row">
          <span class="server-inspector__label">CPU</span>
          <span class="server-inspector__value mono"
            >{{ server.monitor.cpu.used }}/{{ server.monitor.cpu.total }}
            {{ server.monitor.cpu.unit }} · {{ server.monitor.cpu.percent }}%</span
          >
        </div>
        <div class="server-inspector__row">
          <span class="server-inspector__label">{{ t('common.memory') }}</span>
          <span class="server-inspector__value mono"
            >{{ server.monitor.memory.used }}/{{ server.monitor.memory.total }}
            {{ server.monitor.memory.unit }} · {{ server.monitor.memory.percent }}%</span
          >
        </div>
        <div class="server-inspector__row">
          <span class="server-inspector__label">{{ t('server.disk') }}</span>
          <span class="server-inspector__value mono"
            >{{ server.monitor.disk.used }}/{{ server.monitor.disk.total }}
            {{ server.monitor.disk.unit }} · {{ server.monitor.disk.percent }}%</span
          >
        </div>
        <div class="server-inspector__row">
          <span class="server-inspector__label">{{ t('server.metrics.load') }}</span>
          <span class="server-inspector__value mono"
            >{{ server.monitor.load.load1 }} / {{ server.monitor.load.load5 }} /
            {{ server.monitor.load.load15 }}</span
          >
        </div>
      </div>

      <!-- 累计 -->
      <div class="server-inspector__group">
        <div class="server-inspector__group-title">{{ t('server.inspector.cumulative') }}</div>
        <div class="server-inspector__row">
          <span class="server-inspector__label">{{ t('tunnel.metrics.totalUpload') }}</span>
          <span class="server-inspector__value mono">{{
            formatBytes(server.traffic.totalUpload)
          }}</span>
        </div>
        <div class="server-inspector__row">
          <span class="server-inspector__label">{{ t('tunnel.metrics.totalDownload') }}</span>
          <span class="server-inspector__value mono">{{
            formatBytes(server.traffic.totalDownload)
          }}</span>
        </div>
        <div class="server-inspector__row">
          <span class="server-inspector__label">{{ t('server.metrics.uptime') }}</span>
          <span class="server-inspector__value mono">{{
            formatDuration(server.statistics.uptime, t)
          }}</span>
        </div>
        <div class="server-inspector__row">
          <span class="server-inspector__label">{{ t('tunnel.metrics.totalRequests') }}</span>
          <span class="server-inspector__value mono">{{
            formatNumber(server.statistics.requests)
          }}</span>
        </div>
      </div>

      <!-- 健康 -->
      <div class="server-inspector__group">
        <div class="server-inspector__group-title">{{ t('server.detail.health') }}</div>
        <div class="server-inspector__row">
          <span class="server-inspector__label">{{ t('server.overview.status') }}</span>
          <span
            class="server-inspector__value"
            :style="{ color: healthColor(server.health.overall) }">
            {{ healthLabel }}
          </span>
        </div>
        <div class="server-inspector__row">
          <span class="server-inspector__label">{{ t('server.inspector.score') }}</span>
          <span class="server-inspector__value mono">{{ server.health.score }}/100</span>
        </div>
      </div>

      <!-- 标签 -->
      <div v-if="server.tags.length" class="server-inspector__group">
        <div class="server-inspector__group-title">{{ t('server.dialog.tags') }}</div>
        <div class="server-inspector__tags">
          <ServerTag v-for="tag in server.tags" :key="tag" :name="tag" />
        </div>
      </div>

      <!-- 最近日志 -->
      <div class="server-inspector__group">
        <div class="server-inspector__group-title">{{ t('server.inspector.recentLogs') }}</div>
        <div class="server-inspector__logs">
          <div v-for="log in recentLogs" :key="log.id" class="server-inspector__log">
            <span
              class="server-inspector__log-level"
              :class="`server-log-line__level--${log.level}`">
              {{ log.level }}
            </span>
            <span class="server-inspector__log-msg">{{ log.message }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import GIcon from '@components/icons/GIcon.vue'
import ServerStatus from './ServerStatus.vue'
import ServerBadge from './ServerBadge.vue'
import ServerTag from './ServerTag.vue'
import type { Server } from '../types'
import {
  KIND_MAP,
  serverColorVars,
  healthColor,
  formatBytes,
  formatSpeed,
  formatDuration,
  formatNumber,
} from '../utils'

const props = defineProps<{ server: Server }>()
const { t } = useI18n()

const colorVars = computed(() => serverColorVars(props.server.kind))
const kindPreset = computed(() => KIND_MAP[props.server.kind])
const recentLogs = computed(() => props.server.logs.slice(-5).reverse())

const healthLabel = computed(() => {
  return t(`server.healthStatus.${props.server.health.overall}`)
})

/* Mini chart 路径（下载流量） */
const downloadPath = computed(() => buildPath(props.server.traffic.history, 'download'))
const downloadArea = computed(() => {
  const path = buildPath(props.server.traffic.history, 'download')
  if (!path) return ''
  return `${path} L 200 80 L 0 80 Z`
})

function buildPath(history: Server['traffic']['history'], key: 'upload' | 'download'): string {
  if (!history.length) return ''
  const max = Math.max(...history.map((p) => p[key]), 1)
  const stepX = history.length > 1 ? 200 / (history.length - 1) : 0
  return history
    .map((p, i) => {
      const x = i * stepX
      const y = 78 - (p[key] / max) * 70
      return `${i === 0 ? 'M' : 'L'} ${x.toFixed(1)} ${y.toFixed(1)}`
    })
    .join(' ')
}
</script>
