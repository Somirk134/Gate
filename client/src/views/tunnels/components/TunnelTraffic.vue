<!--
  TunnelTraffic — 工作区 Traffic 标签
  ------------------------------------------------------------------
  展示：上传 / 下载 / 累计上传 / 累计下载 / 今日 + 历史趋势 Mini Chart。
-->
<template>
  <div class="tunnel-traffic">
    <div class="tunnel-traffic-grid">
      <div class="tunnel-traffic-card">
        <div class="tunnel-traffic-card__head">
          <span
            class="tunnel-traffic-card__icon"
            style="background:#22C55E22;color:#22C55E"
          >
            <GIcon
              name="arrow-up"
              :size="16"
            />
          </span>
          <span class="tunnel-traffic-card__label">上传速度</span>
        </div>
        <span class="tunnel-traffic-card__value">{{ formatSpeed(tunnel.traffic.uploadSpeed) }}</span>
        <span class="tunnel-traffic-card__sub">今日 {{ formatBytes(tunnel.traffic.todayUpload) }}</span>
      </div>

      <div class="tunnel-traffic-card">
        <div class="tunnel-traffic-card__head">
          <span
            class="tunnel-traffic-card__icon"
            style="background:#5B8DEF22;color:#5B8DEF"
          >
            <GIcon
              name="arrow-down"
              :size="16"
            />
          </span>
          <span class="tunnel-traffic-card__label">下载速度</span>
        </div>
        <span class="tunnel-traffic-card__value">{{ formatSpeed(tunnel.traffic.downloadSpeed) }}</span>
        <span class="tunnel-traffic-card__sub">今日 {{ formatBytes(tunnel.traffic.todayDownload) }}</span>
      </div>

      <div class="tunnel-traffic-card">
        <div class="tunnel-traffic-card__head">
          <span
            class="tunnel-traffic-card__icon"
            style="background:#7C6FF222;color:#7C6FF2"
          >
            <GIcon
              name="cloud-upload"
              :size="16"
            />
          </span>
          <span class="tunnel-traffic-card__label">累计上传</span>
        </div>
        <span class="tunnel-traffic-card__value">{{ formatBytes(tunnel.traffic.totalUpload) }}</span>
        <span class="tunnel-traffic-card__sub">峰值 {{ formatSpeed(tunnel.statistics.peakSpeed) }}</span>
      </div>

      <div class="tunnel-traffic-card">
        <div class="tunnel-traffic-card__head">
          <span
            class="tunnel-traffic-card__icon"
            style="background:#06B6D422;color:#06B6D4"
          >
            <GIcon
              name="cloud-download"
              :size="16"
            />
          </span>
          <span class="tunnel-traffic-card__label">累计下载</span>
        </div>
        <span class="tunnel-traffic-card__value">{{ formatBytes(tunnel.traffic.totalDownload) }}</span>
        <span class="tunnel-traffic-card__sub">总计 {{ formatBytes(totalBytes) }}</span>
      </div>
    </div>

    <!-- 历史趋势 -->
    <div
      class="tunnel-info-card"
      style="margin-top: var(--space-4)"
    >
      <div class="tunnel-info-card__title">
        <GIcon
          name="chart-line"
          :size="12"
        />
        流量趋势（最近 12 个采样点）
      </div>
      <div class="tunnel-traffic__chart-wrap">
        <svg
          class="tunnel-traffic__chart"
          viewBox="0 0 600 180"
          preserveAspectRatio="none"
        >
          <defs>
            <linearGradient
              id="tunnel-traffic-up"
              x1="0"
              y1="0"
              x2="0"
              y2="1"
            >
              <stop
                offset="0%"
                stop-color="#22C55E"
                stop-opacity="0.30"
              />
              <stop
                offset="100%"
                stop-color="#22C55E"
                stop-opacity="0"
              />
            </linearGradient>
            <linearGradient
              id="tunnel-traffic-down"
              x1="0"
              y1="0"
              x2="0"
              y2="1"
            >
              <stop
                offset="0%"
                stop-color="#5B8DEF"
                stop-opacity="0.30"
              />
              <stop
                offset="100%"
                stop-color="#5B8DEF"
                stop-opacity="0"
              />
            </linearGradient>
          </defs>

          <!-- 网格线 -->
          <line
            v-for="i in 3"
            :key="`g-${i}`"
            x1="0"
            :x2="600"
            :y1="i * 45"
            :y2="i * 45"
            stroke="var(--color-border-subtle)"
            stroke-width="1"
          />

          <path
            :d="downArea"
            fill="url(#tunnel-traffic-down)"
          />
          <path
            :d="downPath"
            fill="none"
            stroke="#5B8DEF"
            stroke-width="2"
          />
          <path
            :d="upArea"
            fill="url(#tunnel-traffic-up)"
          />
          <path
            :d="upPath"
            fill="none"
            stroke="#22C55E"
            stroke-width="2"
          />
        </svg>
        <div class="tunnel-traffic__legend">
          <span class="tunnel-traffic__legend-item">
            <span
              class="tunnel-traffic__legend-dot"
              style="background:#5B8DEF"
            />
            下载
          </span>
          <span class="tunnel-traffic__legend-item">
            <span
              class="tunnel-traffic__legend-dot"
              style="background:#22C55E"
            />
            上传
          </span>
        </div>
      </div>
    </div>

    <!-- History 表 -->
    <div
      class="tunnel-info-card"
      style="margin-top: var(--space-4)"
    >
      <div class="tunnel-info-card__title">
        <GIcon
          name="history"
          :size="12"
        />
        历史采样（Mock）
      </div>
      <div class="tunnel-conn-table">
        <div class="tunnel-conn-row tunnel-conn-row--head">
          <span class="tunnel-conn-row__cell">时间</span>
          <span class="tunnel-conn-row__cell">上传</span>
          <span class="tunnel-conn-row__cell">下载</span>
          <span class="tunnel-conn-row__cell">合计</span>
          <span class="tunnel-conn-row__cell">趋势</span>
        </div>
        <div
          v-for="(p, i) in tunnel.traffic.history"
          :key="i"
          class="tunnel-conn-row"
        >
          <span class="tunnel-conn-row__cell mono">{{ p.time }}</span>
          <span class="tunnel-conn-row__cell mono">{{ formatSpeed(p.upload) }}</span>
          <span class="tunnel-conn-row__cell mono">{{ formatSpeed(p.download) }}</span>
          <span class="tunnel-conn-row__cell mono">{{ formatBytes(p.upload + p.download) }}</span>
          <span class="tunnel-conn-row__cell">
            <GIcon
              :name="trendIcon(i)"
              :size="12"
              :style="{ color: trendColor(i) }"
            />
          </span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue"
import GIcon from "@components/icons/GIcon.vue"
import type { Tunnel } from "../types"
import { formatBytes, formatSpeed } from "../utils"

const props = defineProps<{ tunnel: Tunnel }>()

const totalBytes = computed(
  () => props.tunnel.traffic.totalUpload + props.tunnel.traffic.totalDownload,
)

function buildPath(key: "upload" | "download"): string {
  const h = props.tunnel.traffic.history
  if (!h.length) return ""
  const max = Math.max(...h.map((p) => p[key]), 1)
  const stepX = h.length > 1 ? 600 / (h.length - 1) : 0
  return h
    .map((p, i) => {
      const x = i * stepX
      const y = 175 - (p[key] / max) * 160
      return `${i === 0 ? "M" : "L"} ${x.toFixed(1)} ${y.toFixed(1)}`
    })
    .join(" ")
}

function buildArea(key: "upload" | "download"): string {
  const path = buildPath(key)
  if (!path) return ""
  return `${path} L 600 180 L 0 180 Z`
}

const upPath = computed(() => buildPath("upload"))
const upArea = computed(() => buildArea("upload"))
const downPath = computed(() => buildPath("download"))
const downArea = computed(() => buildArea("download"))

function trendIcon(i: number): string {
  if (i === 0) return "minus"
  const h = props.tunnel.traffic.history
  const cur = h[i].upload + h[i].download
  const prev = h[i - 1].upload + h[i - 1].download
  if (cur > prev) return "trending-up"
  if (cur < prev) return "trending-down"
  return "minus"
}

function trendColor(i: number): string {
  const icon = trendIcon(i)
  if (icon === "trending-up") return "var(--color-success)"
  if (icon === "trending-down") return "var(--color-error)"
  return "var(--text-tertiary)"
}
</script>

<style scoped>
.tunnel-traffic__chart-wrap {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.tunnel-traffic__chart {
  width: 100%;
  height: 180px;
  display: block;
}

.tunnel-traffic__legend {
  display: flex;
  align-items: center;
  gap: var(--space-4);
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}

.tunnel-traffic__legend-item {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
}

.tunnel-traffic__legend-dot {
  width: 8px;
  height: 8px;
  border-radius: var(--radius-full);
}
</style>
