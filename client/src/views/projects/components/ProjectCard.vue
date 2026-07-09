<!--
  ProjectCard — 项目卡片
  ------------------------------------------------------------------
  列表页核心卡片。展示项目图标/颜色/名称/描述/隧道/域名/
  证书数量、最近活动、标签和状态。

  交互：
    - 单击进入详情
    - 始终展示操作栏（启动/停止/编辑/删除）
    - Pin / Favorite 快捷切换
    - 右键菜单（预留）

  颜色通过 --project-color CSS 变量驱动，保持与 Sidebar/Dashboard 一致。
-->
<template>
  <div
    class="project-card"
    :class="[`project-card--${project.status}`, { 'project-card--pinned': project.pinned }]"
    :style="colorVars"
    @click="$emit('open', project)"
    @contextmenu.prevent="$emit('contextmenu', project, $event)">
    <!-- 顶部颜色条 -->
    <div class="project-card__accent" />

    <div class="project-card__body">
      <!-- 头部：图标 + 名称 + 状态 + 操作 -->
      <div class="project-card__head">
        <span class="project-card__icon">
          <GIcon :name="project.icon" :size="20" />
        </span>
        <div class="project-card__title-wrap">
          <span class="project-card__name" :title="project.name">{{ project.name }}</span>
          <GStatusBadge :status="statusDotType" :label="statusLabel" size="sm" />
        </div>
        <div class="project-card__quick-actions">
          <button
            class="project-card__quick-btn"
            :class="{ 'project-card__quick-btn--active': project.favorite }"
            :title="t('project.card.favorite')"
            @click.stop="$emit('toggle-favorite', project.id)">
            <GIcon :name="project.favorite ? 'star' : 'star-off'" :size="14" />
          </button>
          <button
            class="project-card__quick-btn"
            :class="{ 'project-card__quick-btn--pinned': project.pinned }"
            :title="t('project.card.pin')"
            @click.stop="$emit('toggle-pin', project.id)">
            <GIcon name="pin" :size="14" />
          </button>
        </div>
      </div>

      <!-- 描述 -->
      <p class="project-card__desc">
        {{ project.description }}
      </p>

      <!-- 标签 -->
      <div v-if="project.tags.length" class="project-card__tags">
        <ProjectTag v-for="tag in project.tags.slice(0, 3)" :key="tag" :name="tag" />
        <span v-if="project.tags.length > 3" class="project-card__tag-more">
          +{{ project.tags.length - 3 }}
        </span>
      </div>

      <!-- 指标 -->
      <div class="project-card__metrics">
        <div class="project-card__metric">
          <GIcon name="link" :size="12" />
          <span class="project-card__metric-value">{{ project.tunnelCount }}</span>
          <span class="project-card__metric-label">{{ t('project.card.tunnel') }}</span>
        </div>
        <div class="project-card__metric project-card__metric--running">
          <span class="project-card__metric-dot" />
          <span class="project-card__metric-value">{{ project.runningTunnelCount }}</span>
          <span class="project-card__metric-label">{{ t('project.card.running') }}</span>
        </div>
        <div class="project-card__metric">
          <GIcon name="globe" :size="12" />
          <span class="project-card__metric-value">{{ project.domainCount }}</span>
          <span class="project-card__metric-label">{{ t('project.card.domain') }}</span>
        </div>
        <div class="project-card__metric">
          <GIcon name="shield-check" :size="12" />
          <span class="project-card__metric-value">{{ project.certificateCount }}</span>
          <span class="project-card__metric-label">{{ t('project.card.certificate') }}</span>
        </div>
      </div>

      <!-- 时间信息 -->
      <div class="project-card__time">
        <span class="project-card__time-item">
          <GIcon name="clock" :size="11" />
          {{ project.lastStartedAt }}
        </span>
        <span class="project-card__time-item">
          <GIcon name="calendar" :size="11" />
          {{ createdLabel }}
        </span>
      </div>
    </div>

    <!-- 操作栏 -->
    <div class="project-card__actions">
      <GButton
        v-if="!isRunning"
        size="sm"
        variant="primary"
        icon="play"
        @click.stop="$emit('start', project)">
        {{ t('project.card.start') }}
      </GButton>
      <GButton
        v-else
        size="sm"
        variant="secondary"
        icon="stop"
        @click.stop="$emit('stop', project)">
        {{ t('project.card.stop') }}
      </GButton>
      <GButton size="sm" variant="ghost" icon="edit" @click.stop="$emit('edit', project)">
        {{ t('project.card.edit') }}
      </GButton>
      <GIconButton
        name="trash"
        size="sm"
        variant="ghost"
        :title="t('project.card.delete')"
        @click.stop="$emit('delete', project)" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import GIcon from '@components/icons/GIcon.vue'
import GButton from '@components/base/GButton.vue'
import GIconButton from '@components/base/GIconButton.vue'
import GStatusBadge from '@components/status/GStatusBadge.vue'
import ProjectTag from './ProjectTag.vue'
import type { Project } from '../types'
import { projectColorVars } from '../utils'

const props = defineProps<{ project: Project }>()
const { t } = useI18n()

defineEmits<{
  open: [project: Project]
  edit: [project: Project]
  start: [project: Project]
  stop: [project: Project]
  delete: [project: Project]
  'toggle-pin': [id: string]
  'toggle-favorite': [id: string]
  contextmenu: [project: Project, event: MouseEvent]
}>()

const colorVars = computed(() => projectColorVars(props.project.color))

const statusLabel = computed(() => t(`project.statusLabels.${props.project.status}`))

// GStatusDot 接受的 status 类型映射
const statusDotType = computed(() => {
  const map: Record<
    string,
    'online' | 'offline' | 'connecting' | 'starting' | 'error' | 'warning'
  > = {
    running: 'online',
    partial: 'warning',
    stopped: 'offline',
    starting: 'starting',
    error: 'error',
  }
  return map[props.project.status] ?? 'offline'
})

const isRunning = computed(
  () => props.project.status === 'running' || props.project.status === 'partial',
)

const createdLabel = computed(() => {
  const d = new Date(props.project.createdAt)
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')}`
})
</script>

<style scoped>
.project-card {
  position: relative;
  display: flex;
  flex-direction: column;
  background: var(--bg-card);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-card);
  overflow: hidden;
  cursor: pointer;
  transition:
    border-color var(--duration-base) var(--ease-out),
    background-color var(--duration-base) var(--ease-out),
    box-shadow var(--duration-base) var(--ease-out),
    transform var(--duration-base) var(--ease-out);
}

.project-card:hover {
  background: var(--bg-surface-hover);
  border-color: var(--project-color, var(--color-border-strong));
  box-shadow: var(--shadow-hover);
  transform: translateY(-2px);
}

.project-card:active {
  transform: translateY(0);
}

/* 固定项目标记 */
.project-card--pinned {
  border-color: color-mix(in srgb, var(--project-color) 50%, var(--color-border));
}

/* 顶部颜色条 */
.project-card__accent {
  height: 3px;
  background: var(--project-color);
  opacity: 0.9;
}

.project-card__body {
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
  padding: var(--space-4);
}

/* ── 头部 ── */
.project-card__head {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}

.project-card__icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  border-radius: var(--radius-md);
  background: var(--project-color-muted);
  color: var(--project-color);
  flex-shrink: 0;
}

.project-card__title-wrap {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 3px;
}

.project-card__name {
  font-size: var(--text-md);
  font-weight: var(--weight-semibold);
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.project-card__quick-actions {
  display: flex;
  align-items: center;
  gap: 2px;
  opacity: 0.4;
  transition: opacity var(--duration-fast) var(--ease-out);
}

.project-card:hover .project-card__quick-actions {
  opacity: 1;
}

.project-card__quick-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 26px;
  height: 26px;
  border: none;
  background: transparent;
  color: var(--text-tertiary);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--duration-fast) var(--ease-out);
}

.project-card__quick-btn:hover {
  background: var(--bg-surface-active);
  color: var(--text-secondary);
}

.project-card__quick-btn--active {
  color: var(--color-warning);
}

.project-card__quick-btn--pinned {
  color: var(--color-primary);
}

/* ── 描述 ── */
.project-card__desc {
  font-size: var(--text-sm);
  color: var(--text-tertiary);
  line-height: var(--leading-relaxed);
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
  min-height: 36px;
}

/* ── 标签 ── */
.project-card__tags {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: var(--space-1);
}

.project-card__tag-more {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  padding-left: var(--space-1);
}

/* ── 指标 ── */
.project-card__metrics {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: var(--space-3);
  padding-top: var(--space-3);
  border-top: 1px solid var(--border-subtle);
}

.project-card__metric {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  min-width: 0;
}

.project-card__metric-value {
  font-weight: var(--weight-semibold);
  color: var(--text-secondary);
  font-variant-numeric: tabular-nums;
}

.project-card__metric-label {
  color: var(--text-tertiary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 90px;
}

.project-card__metric--running .project-card__metric-value {
  color: var(--color-success);
}

.project-card__metric-dot {
  width: 6px;
  height: 6px;
  border-radius: var(--radius-full);
  background: var(--color-success);
  flex-shrink: 0;
}

/* ── 时间 ── */
.project-card__time {
  display: flex;
  align-items: center;
  gap: var(--space-4);
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}

.project-card__time-item {
  display: inline-flex;
  align-items: center;
  gap: 4px;
}

/* ── 操作栏 ── */
.project-card__actions {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-3) var(--space-4) var(--space-4);
  border-top: 1px solid var(--border-subtle);
}
</style>
