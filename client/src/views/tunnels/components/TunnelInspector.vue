<!--
  TunnelInspector — 右侧实时检查器
  ------------------------------------------------------------------
  实时展示选中隧道：基础信息 / 实时统计 / 最近日志 / 属性。
  未来扩展：CPU / Memory / Network。
-->
<template>
  <div class="tunnel-inspector" :style="colorVars">
    <header class="tunnel-inspector__header">
      <GIcon name="activity" :size="14" />
      <span>{{ t('tunnel.inspector.title') }}</span>
      <span class="tunnel-inspector__live">
        <span class="tunnel-inspector__live-dot" />
        LIVE
      </span>
    </header>

    <div class="tunnel-inspector__body">
      <!-- Hero -->
      <div class="tunnel-inspector__hero">
        <span class="tunnel-inspector__hero-icon">
          <GIcon :name="protocolPreset.icon" :size="24" />
        </span>
        <div class="tunnel-inspector__hero-text">
          <span class="tunnel-inspector__hero-name" :title="tunnel.name">{{ tunnel.name }}</span>
          <TunnelStatus :status="tunnel.status" size="sm" />
        </div>
      </div>

      <!-- 实时统计 -->
      <div class="tunnel-inspector__group">
        <div class="tunnel-inspector__group-title">{{ t('tunnel.inspector.realtimeStats') }}</div>
        <div class="tunnel-inspector__stat">
          <div class="tunnel-inspector__stat-item">
            <GIcon name="arrow-up" :size="12" />
            <span class="tunnel-inspector__stat-label">{{ t('tunnel.metrics.upload') }}</span>
            <span class="tunnel-inspector__stat-value">{{
              formatSpeed(tunnel.traffic.uploadSpeed)
            }}</span>
          </div>
          <div class="tunnel-inspector__stat-item">
            <GIcon name="arrow-down" :size="12" />
            <span class="tunnel-inspector__stat-label">{{ t('tunnel.metrics.download') }}</span>
            <span class="tunnel-inspector__stat-value">{{
              formatSpeed(tunnel.traffic.downloadSpeed)
            }}</span>
          </div>
          <div class="tunnel-inspector__stat-item">
            <GIcon name="link" :size="12" />
            <span class="tunnel-inspector__stat-label">{{ t('tunnel.metrics.connections') }}</span>
            <span class="tunnel-inspector__stat-value">{{ tunnel.statistics.connections }}</span>
          </div>
          <div class="tunnel-inspector__stat-item">
            <GIcon name="gauge" :size="12" />
            <span class="tunnel-inspector__stat-label">{{ t('tunnel.metrics.latency') }}</span>
            <span class="tunnel-inspector__stat-value">{{ tunnel.statistics.avgLatency }}ms</span>
          </div>
        </div>
      </div>

      <!-- Mini Chart -->
      <div class="tunnel-inspector__group">
        <div class="tunnel-inspector__group-title">{{ t('tunnel.inspector.trafficTrend') }}</div>
        <svg class="tunnel-mini-chart" viewBox="0 0 200 80" preserveAspectRatio="none">
          <defs>
            <linearGradient :id="`grad-${tunnel.id}`" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" :stop-color="protocolPreset.color" stop-opacity="0.35" />
              <stop offset="100%" :stop-color="protocolPreset.color" stop-opacity="0" />
            </linearGradient>
          </defs>
          <path :d="downloadPath" fill="none" :stroke="protocolPreset.color" stroke-width="1.5" />
          <path :d="downloadArea" :fill="`url(#grad-${tunnel.id})`" />
        </svg>
      </div>

      <!-- 基础信息 -->
      <div class="tunnel-inspector__group">
        <div class="tunnel-inspector__group-title">{{ t('tunnel.inspector.basic') }}</div>
        <div class="tunnel-inspector__row">
          <span class="tunnel-inspector__label">{{ t('tunnel.overview.protocol') }}</span>
          <TunnelBadge :protocol="tunnel.protocol" size="sm" />
        </div>
        <div class="tunnel-inspector__row">
          <span class="tunnel-inspector__label">{{ t('tunnel.overview.localAddress') }}</span>
          <span class="tunnel-inspector__value mono"
            >{{ tunnel.localHost }}:{{ tunnel.localPort }}</span
          >
        </div>
        <div class="tunnel-inspector__row">
          <span class="tunnel-inspector__label">{{ t('tunnel.overview.remotePort') }}</span>
          <span class="tunnel-inspector__value mono">{{ tunnel.remotePort }}</span>
        </div>
        <div class="tunnel-inspector__row">
          <span class="tunnel-inspector__label">{{ t('tunnel.overview.server') }}</span>
          <span class="tunnel-inspector__value">{{ tunnel.serverName }}</span>
        </div>
        <div class="tunnel-inspector__row">
          <span class="tunnel-inspector__label">{{ t('tunnel.overview.project') }}</span>
          <span class="tunnel-inspector__value">{{ tunnel.projectName }}</span>
        </div>
        <div class="tunnel-inspector__row">
          <span class="tunnel-inspector__label">{{ t('tunnel.overview.autoStart') }}</span>
          <span class="tunnel-inspector__value">
            <GIcon
              :name="tunnel.autoStart ? 'check' : 'close'"
              :size="12"
              :class="tunnel.autoStart ? 'on' : 'off'" />
            {{ tunnel.autoStart ? t('common.enabled') : t('common.disabled') }}
          </span>
        </div>
      </div>

      <!-- 累计 -->
      <div class="tunnel-inspector__group">
        <div class="tunnel-inspector__group-title">{{ t('tunnel.inspector.cumulative') }}</div>
        <div class="tunnel-inspector__row">
          <span class="tunnel-inspector__label">{{ t('tunnel.metrics.totalUpload') }}</span>
          <span class="tunnel-inspector__value mono">{{
            formatBytes(tunnel.traffic.totalUpload)
          }}</span>
        </div>
        <div class="tunnel-inspector__row">
          <span class="tunnel-inspector__label">{{ t('tunnel.metrics.totalDownload') }}</span>
          <span class="tunnel-inspector__value mono">{{
            formatBytes(tunnel.traffic.totalDownload)
          }}</span>
        </div>
        <div class="tunnel-inspector__row">
          <span class="tunnel-inspector__label">{{ t('tunnel.metrics.uptime') }}</span>
          <span class="tunnel-inspector__value mono">{{
            formatDuration(tunnel.statistics.uptime, t)
          }}</span>
        </div>
        <div class="tunnel-inspector__row">
          <span class="tunnel-inspector__label">{{ t('tunnel.metrics.totalRequests') }}</span>
          <span class="tunnel-inspector__value mono">{{
            formatNumber(tunnel.statistics.requests)
          }}</span>
        </div>
      </div>

      <!-- 标签 -->
      <div v-if="tunnel.tags.length" class="tunnel-inspector__group">
        <div class="tunnel-inspector__group-title">{{ t('tunnel.overview.tags') }}</div>
        <div class="tunnel-inspector__tags">
          <TunnelTag v-for="tag in tunnel.tags" :key="tag" :name="tag" />
        </div>
      </div>

      <!-- 最近日志 -->
      <div class="tunnel-inspector__group">
        <div class="tunnel-inspector__group-title">{{ t('tunnel.detail.recentLogs') }}</div>
        <div class="tunnel-inspector__logs">
          <div v-for="log in recentLogs" :key="log.id" class="tunnel-inspector__log">
            <span
              class="tunnel-inspector__log-level"
              :class="`tunnel-log-line__level--${log.level}`">
              {{ log.level }}
            </span>
            <span class="tunnel-inspector__log-msg">{{ log.message }}</span>
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
import TunnelStatus from './TunnelStatus.vue'
import TunnelBadge from './TunnelBadge.vue'
import TunnelTag from './TunnelTag.vue'
import type { Tunnel } from '../types'
import {
  PROTOCOL_MAP,
  tunnelColorVars,
  formatBytes,
  formatSpeed,
  formatDuration,
  formatNumber,
} from '../utils'

const props = defineProps<{ tunnel: Tunnel }>()
const { t } = useI18n()

const colorVars = computed(() => tunnelColorVars(props.tunnel.protocol))
const protocolPreset = computed(() => PROTOCOL_MAP[props.tunnel.protocol])
const recentLogs = computed(() => props.tunnel.logs.slice(-5).reverse())

/* Mini chart 路径（下载流量） */
const downloadPath = computed(() => buildPath(props.tunnel.traffic.history, 'download'))
const downloadArea = computed(() => {
  const path = buildPath(props.tunnel.traffic.history, 'download')
  if (!path) return ''
  return `${path} L 200 80 L 0 80 Z`
})

function buildPath(history: Tunnel['traffic']['history'], key: 'upload' | 'download'): string {
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

<style scoped>
.tunnel-inspector {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-height: 0;
}

.tunnel-inspector__header {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-3) var(--space-4);
  font-size: var(--text-sm);
  font-weight: var(--weight-semibold);
  color: var(--text-secondary);
  border-bottom: 1px solid var(--color-border-subtle);
  flex-shrink: 0;
}

.tunnel-inspector__live {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  margin-left: auto;
  font-size: 10px;
  font-weight: var(--weight-bold);
  letter-spacing: var(--tracking-wider);
  color: var(--color-success);
}

.tunnel-inspector__live-dot {
  width: 6px;
  height: 6px;
  border-radius: var(--radius-full);
  background: var(--color-success);
  animation: g-pulse 1.5s var(--ease-in-out) infinite;
}

.tunnel-inspector__body {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-4);
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

/* ── Hero ── */
.tunnel-inspector__hero {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding-bottom: var(--space-3);
  border-bottom: 1px solid var(--color-border-subtle);
}

.tunnel-inspector__hero-icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 44px;
  height: 44px;
  border-radius: var(--radius-lg);
  background: var(--tunnel-color-muted);
  color: var(--tunnel-color);
  flex-shrink: 0;
}

.tunnel-inspector__hero-text {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
  min-width: 0;
}

.tunnel-inspector__hero-name {
  font-size: var(--text-md);
  font-weight: var(--weight-semibold);
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* ── 信息组 ── */
.tunnel-inspector__group {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.tunnel-inspector__group-title {
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
  color: var(--text-tertiary);
  text-transform: uppercase;
  letter-spacing: var(--tracking-wider);
}

.tunnel-inspector__row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-2);
  font-size: var(--text-sm);
}

.tunnel-inspector__label {
  color: var(--text-tertiary);
  flex-shrink: 0;
}

.tunnel-inspector__value {
  color: var(--text-secondary);
  font-weight: var(--weight-medium);
  text-align: right;
  min-width: 0;
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
}

.tunnel-inspector__value.mono {
  font-family: var(--font-mono);
  font-weight: var(--weight-regular);
  font-size: var(--text-xs);
}

.tunnel-inspector__value :deep(.on) {
  color: var(--color-success);
}
.tunnel-inspector__value :deep(.off) {
  color: var(--text-tertiary);
}

/* ── 实时统计网格 ── */
.tunnel-inspector__stat {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--space-2);
}

.tunnel-inspector__stat-item {
  display: flex;
  align-items: center;
  gap: var(--space-1);
  padding: var(--space-2) var(--space-3);
  background: var(--bg-surface-hover);
  border-radius: var(--radius-md);
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}

.tunnel-inspector__stat-label {
  flex: 1;
}

.tunnel-inspector__stat-value {
  color: var(--text-primary);
  font-weight: var(--weight-semibold);
  font-variant-numeric: tabular-nums;
  font-family: var(--font-mono);
}

/* ── 标签 ── */
.tunnel-inspector__tags {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-1);
}

/* ── 最近日志 ── */
.tunnel-inspector__logs {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
  max-height: 160px;
  overflow-y: auto;
}

.tunnel-inspector__log {
  display: flex;
  align-items: flex-start;
  gap: var(--space-2);
  padding: var(--space-1) var(--space-2);
  border-radius: var(--radius-sm);
  background: var(--bg-surface-hover);
  font-size: 10px;
  line-height: var(--leading-tight);
}

.tunnel-inspector__log-level {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 38px;
  padding: 1px var(--space-1);
  border-radius: var(--radius-xs);
  font-weight: var(--weight-semibold);
  font-size: 9px;
  flex-shrink: 0;
  text-transform: uppercase;
}

.tunnel-inspector__log-msg {
  color: var(--text-secondary);
  word-break: break-word;
  font-family: var(--font-mono);
}
</style>
