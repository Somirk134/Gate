<!--
  ProjectHeader — 详情页头部
  ------------------------------------------------------------------
  显示：图标 / 名称 / 描述 / 状态 / Server / Tunnel Count / Running Count / Traffic
  按钮：启动全部 / 停止全部 / 创建 Tunnel / 编辑 / 更多
-->
<template>
  <div class="project-header" :style="colorVars">
    <div class="project-header__top">
      <div class="project-header__left">
        <GIconButton name="arrow-left" variant="soft" size="md" tooltip="返回项目列表" @click="$emit('back')" />
        <span class="project-header__icon">
          <GIcon :name="project.icon" :size="24" />
        </span>
        <div class="project-header__info">
          <div class="project-header__title-row">
            <h1 class="project-header__name">{{ project.name }}</h1>
            <GStatusBadge :status="statusDotType" :label="statusLabel" size="md" />
            <button
              class="project-header__quick"
              :class="{ 'project-header__quick--active': project.favorite }"
              title="收藏"
              @click="$emit('toggle-favorite', project.id)"
            >
              <GIcon :name="project.favorite ? 'star' : 'star-off'" :size="15" />
            </button>
            <button
              class="project-header__quick"
              :class="{ 'project-header__quick--pinned': project.pinned }"
              title="固定"
              @click="$emit('toggle-pin', project.id)"
            >
              <GIcon name="pin" :size="15" />
            </button>
          </div>
          <p class="project-header__desc">{{ project.description }}</p>
        </div>
      </div>

      <div class="project-header__actions">
        <GButton
          v-if="!isRunning"
          variant="primary"
          icon="play"
          @click="$emit('start-all')"
        >
          启动全部
        </GButton>
        <GButton
          v-else
          variant="secondary"
          icon="stop"
          @click="$emit('stop-all')"
        >
          停止全部
        </GButton>
        <GButton variant="secondary" icon="plus" @click="$emit('create-tunnel')">
          创建 Tunnel
        </GButton>
        <GButton variant="ghost" icon="edit" @click="$emit('edit')">编辑</GButton>
        <GIconButton name="more-vertical" variant="soft" @click="$emit('more')" />
      </div>
    </div>

    <!-- 指标条 -->
    <div class="project-header__metrics">
      <div class="project-header__metric">
        <GIcon name="servers" :size="13" />
        <span class="project-header__metric-label">服务器</span>
        <span class="project-header__metric-value">{{ project.serverName }}</span>
      </div>
      <span class="project-header__sep" />
      <div class="project-header__metric">
        <GIcon name="link" :size="13" />
        <span class="project-header__metric-label">Tunnel</span>
        <span class="project-header__metric-value">{{ project.tunnelCount }}</span>
      </div>
      <span class="project-header__sep" />
      <div class="project-header__metric project-header__metric--running">
        <span class="project-header__metric-dot" />
        <span class="project-header__metric-label">运行中</span>
        <span class="project-header__metric-value">{{ project.runningTunnelCount }}</span>
      </div>
      <span class="project-header__sep" />
      <div class="project-header__metric">
        <GIcon name="cloud" :size="13" />
        <span class="project-header__metric-label">今日流量</span>
        <span class="project-header__metric-value">{{ trafficLabel }}</span>
      </div>
      <span class="project-header__sep" />
      <div class="project-header__metric">
        <GIcon name="clock" :size="13" />
        <span class="project-header__metric-label">最后启动</span>
        <span class="project-header__metric-value">{{ project.lastStartedAt }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue"
import GIcon from "@components/icons/GIcon.vue"
import GButton from "@components/base/GButton.vue"
import GIconButton from "@components/base/GIconButton.vue"
import GStatusBadge from "@components/status/GStatusBadge.vue"
import type { Project } from "../types"
import { STATUS_CONFIG, projectColorVars, formatBytes } from "../utils"

const props = defineProps<{ project: Project }>()

defineEmits<{
  back: []
  "start-all": []
  "stop-all": []
  "create-tunnel": []
  edit: []
  more: []
  "toggle-pin": [id: string]
  "toggle-favorite": [id: string]
}>()

const colorVars = computed(() => projectColorVars(props.project.color))
const statusConfig = computed(() => STATUS_CONFIG[props.project.status])
const statusLabel = computed(() => statusConfig.value.label)

const statusDotType = computed(() => {
  const map: Record<string, "online" | "offline" | "connecting" | "starting" | "error" | "warning"> = {
    running: "online",
    partial: "warning",
    stopped: "offline",
    starting: "starting",
    error: "error",
  }
  return map[props.project.status] ?? "offline"
})

const isRunning = computed(
  () => props.project.status === "running" || props.project.status === "partial",
)

const trafficLabel = computed(() =>
  formatBytes(props.project.statistics.todayTraffic),
)
</script>

<style scoped>
.project-header {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
  padding: var(--space-4) var(--space-5);
  background: var(--bg-card);
  border: 1px solid var(--color-border);
  border-left: 3px solid var(--project-color);
  border-radius: var(--radius-card);
}

.project-header__top {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-4);
  flex-wrap: wrap;
}

.project-header__left {
  display: flex;
  align-items: flex-start;
  gap: var(--space-3);
  min-width: 0;
  flex: 1;
}

.project-header__icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 48px;
  height: 48px;
  border-radius: var(--radius-lg);
  background: var(--project-color-muted);
  color: var(--project-color);
  flex-shrink: 0;
}

.project-header__info {
  min-width: 0;
  flex: 1;
}

.project-header__title-row {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}

.project-header__name {
  font-size: var(--text-2xl);
  font-weight: var(--weight-bold);
  color: var(--text-primary);
  letter-spacing: var(--tracking-tight);
}

.project-header__quick {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  background: transparent;
  color: var(--text-tertiary);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--duration-fast) var(--ease-out);
}

.project-header__quick:hover {
  background: var(--bg-surface-hover);
  color: var(--text-secondary);
}

.project-header__quick--active {
  color: var(--color-warning);
}

.project-header__quick--pinned {
  color: var(--color-primary);
}

.project-header__desc {
  margin-top: var(--space-1);
  font-size: var(--text-sm);
  color: var(--text-tertiary);
  line-height: var(--leading-relaxed);
}

.project-header__actions {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  flex-shrink: 0;
}

/* ── 指标条 ── */
.project-header__metrics {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: var(--space-3);
  padding-top: var(--space-3);
  border-top: 1px solid var(--border-subtle);
}

.project-header__metric {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}

.project-header__metric-label {
  color: var(--text-tertiary);
}

.project-header__metric-value {
  color: var(--text-secondary);
  font-weight: var(--weight-medium);
  font-variant-numeric: tabular-nums;
}

.project-header__metric--running .project-header__metric-value {
  color: var(--color-success);
}

.project-header__metric-dot {
  width: 6px;
  height: 6px;
  border-radius: var(--radius-full);
  background: var(--color-success);
}

.project-header__sep {
  width: 1px;
  height: 12px;
  background: var(--color-border-subtle);
}
</style>
