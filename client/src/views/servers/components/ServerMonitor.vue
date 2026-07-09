<!--
  ServerMonitor — 工作区 Monitor 标签
  ------------------------------------------------------------------
  实时监控面板：CPU / Memory / Disk / Load / Network / Traffic / Connection。
  采用 Card / Mini Chart / Progress / Circle，不使用复杂图表。
  当前全部 Mock，由 store.tick() 驱动实时刷新。
  未来替换为真实 Rust Server 指标即可。
-->
<template>
  <div class="server-monitor">
    <!-- 顶部实时指标 -->
    <div class="server-stat-grid">
      <div class="server-stat-card">
        <div
          class="server-stat-card__icon"
          style="color:#5B8DEF;background:#5B8DEF1f"
        >
          <GIcon
            name="cpu"
            :size="18"
          />
        </div>
        <div class="server-stat-card__body">
          <span class="server-stat-card__value">{{ server.monitor.cpu.percent }}<span class="server-monitor__unit">%</span></span>
          <span class="server-stat-card__label">CPU 使用率</span>
        </div>
      </div>
      <div class="server-stat-card">
        <div
          class="server-stat-card__icon"
          style="color:#7C6FF2;background:#7C6FF21f"
        >
          <GIcon
            name="memory-stick"
            :size="18"
          />
        </div>
        <div class="server-stat-card__body">
          <span class="server-stat-card__value">{{ server.monitor.memory.percent }}<span class="server-monitor__unit">%</span></span>
          <span class="server-stat-card__label">内存使用率</span>
        </div>
      </div>
      <div class="server-stat-card">
        <div
          class="server-stat-card__icon"
          style="color:#F59E0B;background:#F59E0B1f"
        >
          <GIcon
            name="hard-drive"
            :size="18"
          />
        </div>
        <div class="server-stat-card__body">
          <span class="server-stat-card__value">{{ server.monitor.disk.percent }}<span class="server-monitor__unit">%</span></span>
          <span class="server-stat-card__label">磁盘使用率</span>
        </div>
      </div>
      <div class="server-stat-card">
        <div
          class="server-stat-card__icon"
          style="color:#06B6D4;background:#06B6D41f"
        >
          <GIcon
            name="link"
            :size="18"
          />
        </div>
        <div class="server-stat-card__body">
          <span class="server-stat-card__value">{{ server.monitor.connections.active }}</span>
          <span class="server-stat-card__label">活动连接</span>
        </div>
      </div>
    </div>

    <!-- 实时速度曲线 -->
    <div
      class="server-info-card"
      style="margin-top: var(--space-4)"
    >
      <div class="server-info-card__title">
        <GIcon
          name="chart-line"
          :size="12"
        />
        网络速度（最近 12 个采样点）
        <span class="server-monitor__live">
          <span class="server-monitor__live-dot" />
          LIVE
        </span>
      </div>
      <div class="server-monitor__chart-wrap">
        <svg
          class="server-traffic__chart"
          viewBox="0 0 600 180"
          preserveAspectRatio="none"
        >
          <defs>
            <linearGradient
              id="server-monitor-up"
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
              id="server-monitor-down"
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
          <line
            v-for="i in 3"
            :key="`mg-${i}`"
            x1="0"
            :x2="600"
            :y1="i * 45"
            :y2="i * 45"
            stroke="var(--color-border-subtle)"
            stroke-width="1"
          />
          <path
            :d="downArea"
            fill="url(#server-monitor-down)"
          />
          <path
            :d="downPath"
            fill="none"
            stroke="#5B8DEF"
            stroke-width="2"
          />
          <path
            :d="upArea"
            fill="url(#server-monitor-up)"
          />
          <path
            :d="upPath"
            fill="none"
            stroke="#22C55E"
            stroke-width="2"
          />
        </svg>
        <div class="server-traffic__legend">
          <span class="server-traffic__legend-item">
            <span
              class="server-traffic__legend-dot"
              style="background:#5B8DEF"
            />
            下载 {{ formatSpeed(server.traffic.downloadSpeed) }}
          </span>
          <span class="server-traffic__legend-item">
            <span
              class="server-traffic__legend-dot"
              style="background:#22C55E"
            />
            上传 {{ formatSpeed(server.traffic.uploadSpeed) }}
          </span>
        </div>
      </div>
    </div>

    <!-- 资源占用条 -->
    <div
      class="server-info-card"
      style="margin-top: var(--space-4)"
    >
      <div class="server-info-card__title">
        <GIcon
          name="activity"
          :size="12"
        />
        资源占用
      </div>
      <div class="server-monitor__resource">
        <div
          v-for="r in resources"
          :key="r.label"
          class="server-monitor__resource-item"
        >
          <div class="server-monitor__resource-head">
            <GIcon
              :name="r.icon"
              :size="13"
            />
            <span class="server-monitor__resource-label">{{ r.label }}</span>
            <span class="server-monitor__resource-value">{{ r.value }}</span>
          </div>
          <div class="server-monitor__bar">
            <div
              class="server-monitor__bar-fill"
              :style="{ width: `${r.percent}%`, background: r.color }"
            />
          </div>
        </div>
      </div>
    </div>

    <!-- 负载 -->
    <div
      class="server-info-card"
      style="margin-top: var(--space-4)"
    >
      <div class="server-info-card__title">
        <GIcon
          name="gauge"
          :size="12"
        />
        系统负载
      </div>
      <div class="server-info-row">
        <span class="server-info-row__label">1 分钟</span>
        <span class="server-info-row__value mono">{{ server.monitor.load.load1 }}</span>
      </div>
      <div class="server-info-row">
        <span class="server-info-row__label">5 分钟</span>
        <span class="server-info-row__value mono">{{ server.monitor.load.load5 }}</span>
      </div>
      <div class="server-info-row">
        <span class="server-info-row__label">15 分钟</span>
        <span class="server-info-row__value mono">{{ server.monitor.load.load15 }}</span>
      </div>
      <div class="server-info-row">
        <span class="server-info-row__label">CPU 核心</span>
        <span class="server-info-row__value mono">{{ server.monitor.load.cores }}</span>
      </div>
    </div>

    <p class="server-connection__hint">
      <GIcon
        name="info-circle"
        :size="12"
      />
      当前为 Mock 实时数据，每秒刷新。未来将接入真实 Rust Server 指标。
    </p>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue"
import GIcon from "@components/icons/GIcon.vue"
import type { Server } from "../types"
import { formatSpeed } from "../utils"

const props = defineProps<{ server: Server }>()

function buildPath(key: "upload" | "download"): string {
  const h = props.server.traffic.history
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

const resources = computed(() => [
  {
    label: "CPU",
    icon: "cpu",
    value: `${props.server.monitor.cpu.used}/${props.server.monitor.cpu.total} ${props.server.monitor.cpu.unit}`,
    percent: props.server.monitor.cpu.percent,
    color: "var(--color-primary)",
  },
  {
    label: "Memory",
    icon: "memory-stick",
    value: `${props.server.monitor.memory.used}/${props.server.monitor.memory.total} ${props.server.monitor.memory.unit}`,
    percent: props.server.monitor.memory.percent,
    color: "var(--color-secondary)",
  },
  {
    label: "Disk",
    icon: "hard-drive",
    value: `${props.server.monitor.disk.used}/${props.server.monitor.disk.total} ${props.server.monitor.disk.unit}`,
    percent: props.server.monitor.disk.percent,
    color: "var(--color-warning)",
  },
])
</script>
