<!--
  ProjectStatistics — 项目统计卡片组
  ------------------------------------------------------------------
  展示：今日流量 / 累计流量 / 运行时间 / 连接数 / Tunnel 数量
  全部使用 Card 布局。
-->
<template>
  <section class="project-stat-section">
    <div class="project-section__head">
      <div class="project-section__title">
        <GIcon
          name="chart-bar"
          :size="16"
          class="project-section__title-icon"
        />
        <span>统计概览</span>
      </div>
    </div>
    <div class="project-stat-grid">
      <GCard
        v-for="item in stats"
        :key="item.label"
        variant="plain"
        padding="md"
        class="stat-card"
      >
        <div
          class="stat-card__icon"
          :style="{ color: item.color, background: item.color + '1f' }"
        >
          <GIcon
            :name="item.icon"
            :size="18"
          />
        </div>
        <div class="stat-card__body">
          <span class="stat-card__value">{{ item.value }}</span>
          <span class="stat-card__label">{{ item.label }}</span>
        </div>
      </GCard>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed } from "vue"
import GCard from "@components/base/GCard.vue"
import GIcon from "@components/icons/GIcon.vue"
import type { Project } from "../types"
import { formatBytes, formatDuration } from "../utils"

const props = defineProps<{ project: Project }>()

const stats = computed(() => [
  {
    label: "今日流量",
    value: formatBytes(props.project.statistics.todayTraffic),
    icon: "download",
    color: "#5B8DEF",
  },
  {
    label: "累计流量",
    value: formatBytes(props.project.statistics.totalTraffic),
    icon: "cloud",
    color: "#7C6FF2",
  },
  {
    label: "运行时间",
    value: formatDuration(props.project.statistics.uptime),
    icon: "clock",
    color: "#22C55E",
  },
  {
    label: "连接数",
    value: String(props.project.statistics.connections),
    icon: "link",
    color: "#F59E0B",
  },
  {
    label: "Tunnel 数量",
    value: String(props.project.tunnelCount),
    icon: "router",
    color: "#06B6D4",
  },
])
</script>

<style scoped>
.stat-card :deep(.g-card__body) {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}

.stat-card__icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  border-radius: var(--radius-md);
  flex-shrink: 0;
}

.stat-card__body {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}

.stat-card__value {
  font-size: var(--text-lg);
  font-weight: var(--weight-semibold);
  color: var(--text-primary);
  font-variant-numeric: tabular-nums;
  line-height: 1.2;
}

.stat-card__label {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}
</style>
