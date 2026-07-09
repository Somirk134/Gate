<!--
  StatisticsSection — 统计区块
  ------------------------------------------------------------------
  现代统计卡片：Project/Tunnel/Running/Today Traffic/Online Time。
  不使用复杂图表，采用数字增长动画。
-->
<template>
  <section class="dashboard-section">
    <div class="dashboard-section__head">
      <div class="dashboard-section__title">
        <GIcon
          name="chart-bar"
          :size="16"
          class="dashboard-section__title-icon"
        />
        <span>{{ title }}</span>
      </div>
    </div>

    <div class="dashboard-grid--stats">
      <GCard
        v-for="(stat, i) in statItems"
        :key="stat.key"
        variant="plain"
        padding="md"
        class="statistics__card dashboard-card-lift"
        :class="`stagger-${(i % 6) + 1}`"
      >
        <div class="statistics__head">
          <span
            class="statistics__icon"
            :class="`statistics__icon--${stat.variant}`"
          >
            <GIcon
              :name="stat.icon"
              :size="18"
            />
          </span>
          <span
            v-if="stat.trend"
            class="statistics__trend"
          >
            <GIcon
              :name="stat.trendDir"
              :size="12"
            />
            {{ stat.trend }}
          </span>
        </div>
        <div class="statistics__value">
          {{ stat.display }}
        </div>
        <div class="statistics__label">
          {{ stat.label }}
        </div>
      </GCard>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, ref, watchEffect } from "vue"
import GCard from "@components/base/GCard.vue"
import GIcon from "@components/icons/GIcon.vue"
import type { DashboardStatistics } from "../types"
import { useCountUp } from "../composables/useCountUp"

const props = withDefaults(
  defineProps<{
    statistics: DashboardStatistics | null
    title?: string
  }>(),
  {
    title: "数据统计",
  },
)

// 数字增长动画目标值
const projectTarget = ref(0)
const tunnelTarget = ref(0)
const runningTarget = ref(0)

watchEffect(() => {
  projectTarget.value = props.statistics?.projectCount ?? 0
  tunnelTarget.value = props.statistics?.tunnelCount ?? 0
  runningTarget.value = props.statistics?.runningTunnel ?? 0
})

const projectCount = useCountUp(projectTarget)
const tunnelCount = useCountUp(tunnelTarget)
const runningCount = useCountUp(runningTarget)

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B"
  const gb = bytes / (1024 * 1024 * 1024)
  if (gb >= 1) return `${gb.toFixed(1)} GB`
  const mb = bytes / (1024 * 1024)
  return `${mb.toFixed(0)} MB`
}

function formatDuration(sec: number): string {
  const h = Math.floor(sec / 3600)
  const m = Math.floor((sec % 3600) / 60)
  return `${h}h ${m}m`
}

const statItems = computed(() => {
  const s = props.statistics
  return [
    {
      key: "project",
      label: "项目总数",
      icon: "projects",
      variant: "primary" as const,
      display: String(Math.round(projectCount.value)),
      trend: "+2",
      trendDir: "trending-up" as const,
    },
    {
      key: "tunnel",
      label: "隧道总数",
      icon: "link",
      variant: "info" as const,
      display: String(Math.round(tunnelCount.value)),
      trend: "+5",
      trendDir: "trending-up" as const,
    },
    {
      key: "running",
      label: "运行中隧道",
      icon: "activity",
      variant: "success" as const,
      display: String(Math.round(runningCount.value)),
      trend: "+3",
      trendDir: "trending-up" as const,
    },
    {
      key: "upload",
      label: "今日上传",
      icon: "upload",
      variant: "warning" as const,
      display: s ? formatBytes(s.todayUpload) : "—",
      trend: "12%",
      trendDir: "trending-up" as const,
    },
    {
      key: "download",
      label: "今日下载",
      icon: "download",
      variant: "primary" as const,
      display: s ? formatBytes(s.todayDownload) : "—",
      trend: "8%",
      trendDir: "trending-up" as const,
    },
    {
      key: "online",
      label: "在线时长",
      icon: "timer",
      variant: "info" as const,
      display: s ? formatDuration(s.onlineTime) : "—",
      trend: "",
      trendDir: "trending-up" as const,
    },
  ]
})
</script>

<style scoped>
.statistics__card {
  animation: g-slide-in-up var(--duration-base) var(--ease-out) both;
}
.statistics__head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: var(--space-2);
}
.statistics__icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: var(--radius-md);
  background: var(--bg-surface-hover);
  color: var(--text-tertiary);
}
.statistics__icon--primary {
  background: var(--color-primary-muted);
  color: var(--color-primary);
}
.statistics__icon--success {
  background: var(--color-success-muted);
  color: var(--color-success);
}
.statistics__icon--warning {
  background: var(--color-warning-muted);
  color: var(--color-warning);
}
.statistics__icon--info {
  background: var(--color-info-muted);
  color: var(--color-info);
}
.statistics__value {
  font-size: var(--text-2xl);
  font-weight: var(--weight-semibold);
  color: var(--text-primary);
  font-variant-numeric: tabular-nums;
  line-height: var(--leading-tight);
}
.statistics__label {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  text-transform: uppercase;
  letter-spacing: var(--tracking-wider);
  margin-top: 2px;
}
.statistics__trend {
  display: inline-flex;
  align-items: center;
  gap: 2px;
  font-size: var(--text-xs);
  font-weight: var(--weight-medium);
  color: var(--color-success);
}
</style>
