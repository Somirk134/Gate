<!--
  TunnelMonitor — 工作区 Monitor 标签
  ------------------------------------------------------------------
  实时监控面板：速度 / 连接 / 延迟 / CPU（预留）/ 内存（预留）/ 网络。
  数据来自 Runtime Dashboard；未采集到的指标显示为空值。
-->
<template>
  <div class="tunnel-monitor">
    <!-- 顶部实时指标 -->
    <div class="tunnel-stat-grid">
      <div class="tunnel-stat-card">
        <div class="tunnel-stat-card__icon" style="color: #22c55e; background: #22c55e1f">
          <GIcon name="arrow-up" :size="18" />
        </div>
        <div class="tunnel-stat-card__body">
          <span class="tunnel-stat-card__value">{{ formatSpeed(tunnel.traffic.uploadSpeed) }}</span>
          <span class="tunnel-stat-card__label">上传速度</span>
        </div>
      </div>
      <div class="tunnel-stat-card">
        <div class="tunnel-stat-card__icon" style="color: #5b8def; background: #5b8def1f">
          <GIcon name="arrow-down" :size="18" />
        </div>
        <div class="tunnel-stat-card__body">
          <span class="tunnel-stat-card__value">{{
            formatSpeed(tunnel.traffic.downloadSpeed)
          }}</span>
          <span class="tunnel-stat-card__label">下载速度</span>
        </div>
      </div>
      <div class="tunnel-stat-card">
        <div class="tunnel-stat-card__icon" style="color: #f59e0b; background: #f59e0b1f">
          <GIcon name="link" :size="18" />
        </div>
        <div class="tunnel-stat-card__body">
          <span class="tunnel-stat-card__value">{{ tunnel.statistics.connections }}</span>
          <span class="tunnel-stat-card__label">当前连接</span>
        </div>
      </div>
      <div class="tunnel-stat-card">
        <div class="tunnel-stat-card__icon" style="color: #06b6d4; background: #06b6d41f">
          <GIcon name="gauge" :size="18" />
        </div>
        <div class="tunnel-stat-card__body">
          <span class="tunnel-stat-card__value"
            >{{ tunnel.statistics.avgLatency }}<span class="tunnel-monitor__unit">ms</span></span
          >
          <span class="tunnel-stat-card__label">平均延迟</span>
        </div>
      </div>
    </div>

    <!-- 实时速度曲线 -->
    <div class="tunnel-info-card" style="margin-top: var(--space-4)">
      <div class="tunnel-info-card__title">
        <GIcon name="chart-line" :size="12" />
        实时速度（最近 12 个采样点）
        <span class="tunnel-monitor__live">
          <span class="tunnel-monitor__live-dot" />
          LIVE
        </span>
      </div>
      <div class="tunnel-monitor__chart-wrap">
        <svg class="tunnel-traffic__chart" viewBox="0 0 600 180" preserveAspectRatio="none">
          <defs>
            <linearGradient id="tunnel-monitor-up" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stop-color="#22C55E" stop-opacity="0.30" />
              <stop offset="100%" stop-color="#22C55E" stop-opacity="0" />
            </linearGradient>
            <linearGradient id="tunnel-monitor-down" x1="0" y1="0" x2="0" y2="1">
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
          <path :d="downArea" fill="url(#tunnel-monitor-down)" />
          <path :d="downPath" fill="none" stroke="#5B8DEF" stroke-width="2" />
          <path :d="upArea" fill="url(#tunnel-monitor-up)" />
          <path :d="upPath" fill="none" stroke="#22C55E" stroke-width="2" />
        </svg>
        <div class="tunnel-traffic__legend">
          <span class="tunnel-traffic__legend-item">
            <span class="tunnel-traffic__legend-dot" style="background: #5b8def" />
            下载
          </span>
          <span class="tunnel-traffic__legend-item">
            <span class="tunnel-traffic__legend-dot" style="background: #22c55e" />
            上传
          </span>
        </div>
      </div>
    </div>

    <!-- 资源占用（预留） -->
    <div class="tunnel-info-card" style="margin-top: var(--space-4)">
      <div class="tunnel-info-card__title">
        <GIcon name="cpu" :size="12" />
        资源占用
        <GBadge variant="neutral" type="soft" size="sm"> 预留 </GBadge>
      </div>
      <div class="tunnel-monitor__resource">
        <div v-for="r in resources" :key="r.label" class="tunnel-monitor__resource-item">
          <div class="tunnel-monitor__resource-head">
            <GIcon :name="r.icon" :size="13" />
            <span class="tunnel-monitor__resource-label">{{ r.label }}</span>
            <span class="tunnel-monitor__resource-value">{{ r.value }}</span>
          </div>
          <div class="tunnel-monitor__bar">
            <div
              class="tunnel-monitor__bar-fill"
              :style="{ width: `${r.percent}%`, background: r.color }" />
          </div>
        </div>
      </div>
    </div>

    <p class="tunnel-connection__hint">
      <GIcon name="info-circle" :size="12" />
      监控数据来自 Runtime Dashboard；暂无采样时显示空曲线。
    </p>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import GIcon from '@components/icons/GIcon.vue'
import GBadge from '@components/base/GBadge.vue'
import type { Tunnel } from '../types'
import { formatSpeed } from '../utils'

const props = defineProps<{ tunnel: Tunnel }>()

function buildPath(key: 'upload' | 'download'): string {
  const h = props.tunnel.traffic.history
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
  { label: 'CPU', icon: 'cpu', value: '—', percent: 0, color: 'var(--color-primary)' },
  { label: '内存', icon: 'memory-stick', value: '—', percent: 0, color: 'var(--color-secondary)' },
  {
    label: '网络',
    icon: 'network',
    value: formatSpeed(props.tunnel.traffic.downloadSpeed),
    percent: props.tunnel.traffic.downloadSpeed > 0 ? 100 : 0,
    color: 'var(--color-success)',
  },
])
</script>

<style scoped>
.tunnel-monitor__unit {
  font-size: var(--text-sm);
  color: var(--text-tertiary);
  margin-left: 2px;
}

.tunnel-monitor__live {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  margin-left: auto;
  font-size: 10px;
  font-weight: var(--weight-bold);
  letter-spacing: var(--tracking-wider);
  color: var(--color-success);
}

.tunnel-monitor__live-dot {
  width: 6px;
  height: 6px;
  border-radius: var(--radius-full);
  background: var(--color-success);
  animation: g-pulse 1.5s var(--ease-in-out) infinite;
}

.tunnel-monitor__chart-wrap {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.tunnel-monitor__resource {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
}

.tunnel-monitor__resource-item {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.tunnel-monitor__resource-head {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  font-size: var(--text-sm);
  color: var(--text-secondary);
}

.tunnel-monitor__resource-label {
  flex: 1;
}

.tunnel-monitor__resource-value {
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}

.tunnel-monitor__bar {
  height: 6px;
  background: var(--bg-surface-hover);
  border-radius: var(--radius-full);
  overflow: hidden;
}

.tunnel-monitor__bar-fill {
  height: 100%;
  border-radius: var(--radius-full);
  transition: width var(--duration-base) var(--ease-out);
}

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
</style>
