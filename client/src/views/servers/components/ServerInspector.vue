<!--
  ServerInspector — 右侧实时检查器
  ------------------------------------------------------------------
  实时展示选中服务器：基础信息 / 实时统计 / 资源 / 流量趋势 / 属性。
-->
<template>
  <div
    class="server-inspector"
    :style="colorVars"
  >
    <header class="server-inspector__header">
      <GIcon
        name="activity"
        :size="14"
      />
      <span>实时检查器</span>
      <span class="server-inspector__live">
        <span class="server-inspector__live-dot" />
        LIVE
      </span>
    </header>

    <div class="server-inspector__body">
      <!-- Hero -->
      <div class="server-inspector__hero">
        <span class="server-inspector__hero-icon">
          <GIcon
            :name="kindPreset.icon"
            :size="24"
          />
        </span>
        <div class="server-inspector__hero-text">
          <span
            class="server-inspector__hero-name"
            :title="server.name"
          >{{ server.name }}</span>
          <ServerStatus
            :status="server.status"
            size="sm"
          />
        </div>
      </div>

      <!-- 实时统计 -->
      <div class="server-inspector__group">
        <div class="server-inspector__group-title">
          实时统计
        </div>
        <div class="server-inspector__stat">
          <div class="server-inspector__stat-item">
            <GIcon
              name="arrow-up"
              :size="12"
            />
            <span class="server-inspector__stat-label">上传</span>
            <span class="server-inspector__stat-value">{{ formatSpeed(server.traffic.uploadSpeed) }}</span>
          </div>
          <div class="server-inspector__stat-item">
            <GIcon
              name="arrow-down"
              :size="12"
            />
            <span class="server-inspector__stat-label">下载</span>
            <span class="server-inspector__stat-value">{{ formatSpeed(server.traffic.downloadSpeed) }}</span>
          </div>
          <div class="server-inspector__stat-item">
            <GIcon
              name="cpu"
              :size="12"
            />
            <span class="server-inspector__stat-label">CPU</span>
            <span class="server-inspector__stat-value">{{ server.monitor.cpu.percent }}%</span>
          </div>
          <div class="server-inspector__stat-item">
            <GIcon
              name="memory-stick"
              :size="12"
            />
            <span class="server-inspector__stat-label">内存</span>
            <span class="server-inspector__stat-value">{{ server.monitor.memory.percent }}%</span>
          </div>
        </div>
      </div>

      <!-- Mini Chart -->
      <div class="server-inspector__group">
        <div class="server-inspector__group-title">
          流量趋势
        </div>
        <svg
          class="server-mini-chart"
          viewBox="0 0 200 80"
          preserveAspectRatio="none"
        >
          <defs>
            <linearGradient
              :id="`grad-${server.id}`"
              x1="0"
              y1="0"
              x2="0"
              y2="1"
            >
              <stop
                offset="0%"
                :stop-color="kindPreset.color"
                stop-opacity="0.35"
              />
              <stop
                offset="100%"
                :stop-color="kindPreset.color"
                stop-opacity="0"
              />
            </linearGradient>
          </defs>
          <path
            :d="downloadPath"
            fill="none"
            :stroke="kindPreset.color"
            stroke-width="1.5"
          />
          <path
            :d="downloadArea"
            :fill="`url(#grad-${server.id})`"
          />
        </svg>
      </div>

      <!-- 基础信息 -->
      <div class="server-inspector__group">
        <div class="server-inspector__group-title">
          基础信息
        </div>
        <div class="server-inspector__row">
          <span class="server-inspector__label">类型</span>
          <ServerBadge
            :kind="server.kind"
            size="sm"
          />
        </div>
        <div class="server-inspector__row">
          <span class="server-inspector__label">公网 IP</span>
          <span class="server-inspector__value mono">{{ server.publicIp }}</span>
        </div>
        <div class="server-inspector__row">
          <span class="server-inspector__label">地区</span>
          <span class="server-inspector__value">{{ server.region }}</span>
        </div>
        <div class="server-inspector__row">
          <span class="server-inspector__label">版本</span>
          <span class="server-inspector__value mono">{{ server.version }}</span>
        </div>
        <div class="server-inspector__row">
          <span class="server-inspector__label">连接方式</span>
          <span class="server-inspector__value mono">{{ server.connectionMethod.toUpperCase() }}</span>
        </div>
        <div class="server-inspector__row">
          <span class="server-inspector__label">自动连接</span>
          <span class="server-inspector__value">
            <GIcon
              :name="server.settings.autoConnect ? 'check' : 'close'"
              :size="12"
              :class="server.settings.autoConnect ? 'on' : 'off'"
            />
            {{ server.settings.autoConnect ? "已启用" : "未启用" }}
          </span>
        </div>
      </div>

      <!-- 资源 -->
      <div class="server-inspector__group">
        <div class="server-inspector__group-title">
          资源占用
        </div>
        <div class="server-inspector__row">
          <span class="server-inspector__label">CPU</span>
          <span class="server-inspector__value mono">{{ server.monitor.cpu.used }}/{{ server.monitor.cpu.total }} {{ server.monitor.cpu.unit }} · {{ server.monitor.cpu.percent }}%</span>
        </div>
        <div class="server-inspector__row">
          <span class="server-inspector__label">内存</span>
          <span class="server-inspector__value mono">{{ server.monitor.memory.used }}/{{ server.monitor.memory.total }} {{ server.monitor.memory.unit }} · {{ server.monitor.memory.percent }}%</span>
        </div>
        <div class="server-inspector__row">
          <span class="server-inspector__label">磁盘</span>
          <span class="server-inspector__value mono">{{ server.monitor.disk.used }}/{{ server.monitor.disk.total }} {{ server.monitor.disk.unit }} · {{ server.monitor.disk.percent }}%</span>
        </div>
        <div class="server-inspector__row">
          <span class="server-inspector__label">负载</span>
          <span class="server-inspector__value mono">{{ server.monitor.load.load1 }} / {{ server.monitor.load.load5 }} / {{ server.monitor.load.load15 }}</span>
        </div>
      </div>

      <!-- 累计 -->
      <div class="server-inspector__group">
        <div class="server-inspector__group-title">
          累计
        </div>
        <div class="server-inspector__row">
          <span class="server-inspector__label">累计上传</span>
          <span class="server-inspector__value mono">{{ formatBytes(server.traffic.totalUpload) }}</span>
        </div>
        <div class="server-inspector__row">
          <span class="server-inspector__label">累计下载</span>
          <span class="server-inspector__value mono">{{ formatBytes(server.traffic.totalDownload) }}</span>
        </div>
        <div class="server-inspector__row">
          <span class="server-inspector__label">运行时长</span>
          <span class="server-inspector__value mono">{{ formatDuration(server.statistics.uptime) }}</span>
        </div>
        <div class="server-inspector__row">
          <span class="server-inspector__label">累计请求</span>
          <span class="server-inspector__value mono">{{ formatNumber(server.statistics.requests) }}</span>
        </div>
      </div>

      <!-- 健康 -->
      <div class="server-inspector__group">
        <div class="server-inspector__group-title">
          健康
        </div>
        <div class="server-inspector__row">
          <span class="server-inspector__label">状态</span>
          <span
            class="server-inspector__value"
            :style="{ color: healthColor(server.health.overall) }"
          >
            {{ healthLabel }}
          </span>
        </div>
        <div class="server-inspector__row">
          <span class="server-inspector__label">得分</span>
          <span class="server-inspector__value mono">{{ server.health.score }}/100</span>
        </div>
      </div>

      <!-- 标签 -->
      <div
        v-if="server.tags.length"
        class="server-inspector__group"
      >
        <div class="server-inspector__group-title">
          标签
        </div>
        <div class="server-inspector__tags">
          <ServerTag
            v-for="tag in server.tags"
            :key="tag"
            :name="tag"
          />
        </div>
      </div>

      <!-- 最近日志 -->
      <div class="server-inspector__group">
        <div class="server-inspector__group-title">
          最近日志
        </div>
        <div class="server-inspector__logs">
          <div
            v-for="log in recentLogs"
            :key="log.id"
            class="server-inspector__log"
          >
            <span
              class="server-inspector__log-level"
              :class="`server-log-line__level--${log.level}`"
            >
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
import { computed } from "vue"
import GIcon from "@components/icons/GIcon.vue"
import ServerStatus from "./ServerStatus.vue"
import ServerBadge from "./ServerBadge.vue"
import ServerTag from "./ServerTag.vue"
import type { Server } from "../types"
import {
  KIND_MAP,
  serverColorVars,
  healthColor,
  formatBytes,
  formatSpeed,
  formatDuration,
  formatNumber,
} from "../utils"

const props = defineProps<{ server: Server }>()

const colorVars = computed(() => serverColorVars(props.server.kind))
const kindPreset = computed(() => KIND_MAP[props.server.kind])
const recentLogs = computed(() => props.server.logs.slice(-5).reverse())

const healthLabel = computed(() => {
  switch (props.server.health.overall) {
    case "healthy": return "健康"
    case "warning": return "警告"
    case "critical": return "严重"
    default: return "未知"
  }
})

/* Mini chart 路径（下载流量） */
const downloadPath = computed(() => buildPath(props.server.traffic.history, "download"))
const downloadArea = computed(() => {
  const path = buildPath(props.server.traffic.history, "download")
  if (!path) return ""
  return `${path} L 200 80 L 0 80 Z`
})

function buildPath(history: Server["traffic"]["history"], key: "upload" | "download"): string {
  if (!history.length) return ""
  const max = Math.max(...history.map((p) => p[key]), 1)
  const stepX = history.length > 1 ? 200 / (history.length - 1) : 0
  return history
    .map((p, i) => {
      const x = i * stepX
      const y = 78 - (p[key] / max) * 70
      return `${i === 0 ? "M" : "L"} ${x.toFixed(1)} ${y.toFixed(1)}`
    })
    .join(" ")
}
</script>
