<!--
  ServerTraffic — 工作区 Traffic 标签
  ------------------------------------------------------------------
  展示：Today / Week / Month / Total / Upload / Download。
  历史趋势 Mini Chart。全部 Mock。
-->
<template>
  <div class="server-traffic">
    <!-- 流量卡片组 -->
    <div class="server-traffic-grid">
      <div class="server-traffic-card">
        <div class="server-traffic-card__head">
          <span class="server-traffic-card__icon" style="background: #22c55e22; color: #22c55e">
            <GIcon name="arrow-up" :size="16" />
          </span>
          <span class="server-traffic-card__label">上传速度</span>
        </div>
        <span class="server-traffic-card__value">{{
          formatSpeed(server.traffic.uploadSpeed)
        }}</span>
        <span class="server-traffic-card__sub"
          >今日 {{ formatBytes(server.traffic.todayUpload) }}</span
        >
      </div>

      <div class="server-traffic-card">
        <div class="server-traffic-card__head">
          <span class="server-traffic-card__icon" style="background: #5b8def22; color: #5b8def">
            <GIcon name="arrow-down" :size="16" />
          </span>
          <span class="server-traffic-card__label">下载速度</span>
        </div>
        <span class="server-traffic-card__value">{{
          formatSpeed(server.traffic.downloadSpeed)
        }}</span>
        <span class="server-traffic-card__sub"
          >今日 {{ formatBytes(server.traffic.todayDownload) }}</span
        >
      </div>

      <div class="server-traffic-card">
        <div class="server-traffic-card__head">
          <span class="server-traffic-card__icon" style="background: #7c6ff222; color: #7c6ff2">
            <GIcon name="cloud-upload" :size="16" />
          </span>
          <span class="server-traffic-card__label">累计上传</span>
        </div>
        <span class="server-traffic-card__value">{{
          formatBytes(server.traffic.totalUpload)
        }}</span>
        <span class="server-traffic-card__sub"
          >峰值 {{ formatSpeed(server.statistics.peakSpeed) }}</span
        >
      </div>

      <div class="server-traffic-card">
        <div class="server-traffic-card__head">
          <span class="server-traffic-card__icon" style="background: #06b6d422; color: #06b6d4">
            <GIcon name="cloud-download" :size="16" />
          </span>
          <span class="server-traffic-card__label">累计下载</span>
        </div>
        <span class="server-traffic-card__value">{{
          formatBytes(server.traffic.totalDownload)
        }}</span>
        <span class="server-traffic-card__sub">总计 {{ formatBytes(totalBytes) }}</span>
      </div>
    </div>

    <!-- 历史趋势 -->
    <div class="server-info-card" style="margin-top: var(--space-4)">
      <div class="server-info-card__title">
        <GIcon name="chart-line" :size="12" />
        流量趋势（最近 12 个采样点）
      </div>
      <div class="server-traffic__chart-wrap">
        <svg class="server-traffic__chart" viewBox="0 0 600 180" preserveAspectRatio="none">
          <defs>
            <linearGradient id="server-traffic-up" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stop-color="#22C55E" stop-opacity="0.30" />
              <stop offset="100%" stop-color="#22C55E" stop-opacity="0" />
            </linearGradient>
            <linearGradient id="server-traffic-down" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stop-color="#5B8DEF" stop-opacity="0.30" />
              <stop offset="100%" stop-color="#5B8DEF" stop-opacity="0" />
            </linearGradient>
          </defs>
          <line
            v-for="i in 3"
            :key="`g-${i}`"
            x1="0"
            :x2="600"
            :y1="i * 45"
            :y2="i * 45"
            stroke="var(--color-border-subtle)"
            stroke-width="1" />
          <path :d="downArea" fill="url(#server-traffic-down)" />
          <path :d="downPath" fill="none" stroke="#5B8DEF" stroke-width="2" />
          <path :d="upArea" fill="url(#server-traffic-up)" />
          <path :d="upPath" fill="none" stroke="#22C55E" stroke-width="2" />
        </svg>
        <div class="server-traffic__legend">
          <span class="server-traffic__legend-item">
            <span class="server-traffic__legend-dot" style="background: #5b8def" />
            下载
          </span>
          <span class="server-traffic__legend-item">
            <span class="server-traffic__legend-dot" style="background: #22c55e" />
            上传
          </span>
        </div>
      </div>
    </div>

    <!-- 周期统计 -->
    <div class="server-info-card" style="margin-top: var(--space-4)">
      <div class="server-info-card__title">
        <GIcon name="calendar" :size="12" />
        周期统计
      </div>
      <div class="server-conn-table">
        <div class="server-conn-row server-conn-row--head">
          <span class="server-conn-row__cell">周期</span>
          <span class="server-conn-row__cell">上传</span>
          <span class="server-conn-row__cell">下载</span>
          <span class="server-conn-row__cell">合计</span>
          <span class="server-conn-row__cell">占比</span>
        </div>
        <div v-for="row in periodRows" :key="row.label" class="server-conn-row">
          <span class="server-conn-row__cell">{{ row.label }}</span>
          <span class="server-conn-row__cell mono">{{ formatBytes(row.upload) }}</span>
          <span class="server-conn-row__cell mono">{{ formatBytes(row.download) }}</span>
          <span class="server-conn-row__cell mono">{{
            formatBytes(row.upload + row.download)
          }}</span>
          <span class="server-conn-row__cell mono">{{ row.percent }}%</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import GIcon from '@components/icons/GIcon.vue'
import type { Server } from '../types'
import { formatBytes, formatSpeed } from '../utils'

const props = defineProps<{ server: Server }>()

const totalBytes = computed(
  () => props.server.traffic.totalUpload + props.server.traffic.totalDownload,
)

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

const periodRows = computed(() => {
  const total = totalBytes.value || 1
  return [
    {
      label: '今日',
      upload: props.server.traffic.todayUpload,
      download: props.server.traffic.todayDownload,
      percent: Math.round(
        ((props.server.traffic.todayUpload + props.server.traffic.todayDownload) / total) * 100,
      ),
    },
    {
      label: '本周',
      upload: props.server.traffic.weekUpload,
      download: props.server.traffic.weekDownload,
      percent: Math.round(
        ((props.server.traffic.weekUpload + props.server.traffic.weekDownload) / total) * 100,
      ),
    },
    {
      label: '本月',
      upload: props.server.traffic.monthUpload,
      download: props.server.traffic.monthDownload,
      percent: Math.round(
        ((props.server.traffic.monthUpload + props.server.traffic.monthDownload) / total) * 100,
      ),
    },
    {
      label: '累计',
      upload: props.server.traffic.totalUpload,
      download: props.server.traffic.totalDownload,
      percent: 100,
    },
  ]
})
</script>
