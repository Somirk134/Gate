<!--
  ProjectInspector — 详情页右侧检查器
  ------------------------------------------------------------------
  实时展示项目详情：基础信息 / 标签 / 统计 / 操作。
  非 Tunnel 详情，而是项目级别的 Inspector。
-->
<template>
  <div class="project-inspector" :style="colorVars">
    <header class="project-inspector__header">
      <GIcon name="info-circle" :size="14" />
      <span>{{ t('project.inspector.title') }}</span>
    </header>

    <div class="project-inspector__body">
      <!-- 项目图标预览 -->
      <div class="project-inspector__hero">
        <span class="project-inspector__hero-icon">
          <GIcon :name="project.icon" :size="28" />
        </span>
        <div class="project-inspector__hero-text">
          <span class="project-inspector__hero-name">{{ project.name }}</span>
          <GStatusBadge :status="statusDotType" :label="statusLabel" size="sm" />
        </div>
      </div>

      <!-- 基础信息 -->
      <div class="project-inspector__group">
        <div class="project-inspector__group-title">{{ t('project.inspector.basic') }}</div>
        <div class="project-inspector__row">
          <span class="project-inspector__label">{{ t('project.inspector.projectId') }}</span>
          <span class="project-inspector__value mono">{{ project.id }}</span>
        </div>
        <div class="project-inspector__row">
          <span class="project-inspector__label">{{ t('project.inspector.server') }}</span>
          <span class="project-inspector__value">{{ project.serverName }}</span>
        </div>
        <div class="project-inspector__row">
          <span class="project-inspector__label">{{ t('project.inspector.autoStart') }}</span>
          <span class="project-inspector__value">
            <GIcon
              :name="project.autoStart ? 'check' : 'close'"
              :size="12"
              :class="project.autoStart ? 'on' : 'off'" />
            {{ project.autoStart ? t('common.enabled') : t('common.disabled') }}
          </span>
        </div>
        <div class="project-inspector__row">
          <span class="project-inspector__label">{{ t('project.inspector.createdAt') }}</span>
          <span class="project-inspector__value">{{ dateLabel(project.createdAt) }}</span>
        </div>
        <div class="project-inspector__row">
          <span class="project-inspector__label">{{ t('project.inspector.updatedAt') }}</span>
          <span class="project-inspector__value">{{ dateLabel(project.updatedAt) }}</span>
        </div>
      </div>

      <!-- 标签 -->
      <div v-if="project.tags.length" class="project-inspector__group">
        <div class="project-inspector__group-title">{{ t('project.inspector.tags') }}</div>
        <div class="project-inspector__tags">
          <ProjectTag v-for="tag in project.tags" :key="tag" :name="tag" />
        </div>
      </div>

      <!-- 统计 -->
      <div class="project-inspector__group">
        <div class="project-inspector__group-title">{{ t('project.inspector.runtimeStats') }}</div>
        <div class="project-inspector__row">
          <span class="project-inspector__label">{{ t('project.stats.todayTraffic') }}</span>
          <span class="project-inspector__value mono">{{
            formatBytes(project.statistics.todayTraffic)
          }}</span>
        </div>
        <div class="project-inspector__row">
          <span class="project-inspector__label">{{ t('project.stats.totalTraffic') }}</span>
          <span class="project-inspector__value mono">{{
            formatBytes(project.statistics.totalTraffic)
          }}</span>
        </div>
        <div class="project-inspector__row">
          <span class="project-inspector__label">{{ t('project.stats.uptime') }}</span>
          <span class="project-inspector__value mono">{{
            formatDuration(project.statistics.uptime)
          }}</span>
        </div>
        <div class="project-inspector__row">
          <span class="project-inspector__label">{{ t('project.stats.connections') }}</span>
          <span class="project-inspector__value">{{ project.statistics.connections }}</span>
        </div>
      </div>

      <!-- 备注 -->
      <div v-if="project.remark" class="project-inspector__group">
        <div class="project-inspector__group-title">{{ t('project.inspector.remark') }}</div>
        <p class="project-inspector__remark">
          {{ project.remark }}
        </p>
      </div>

      <!-- 操作 -->
      <div class="project-inspector__actions">
        <GButton
          v-if="!isRunning"
          variant="primary"
          icon="play"
          block
          @click="$emit('start', project.id)">
          {{ t('project.inspector.startProject') }}
        </GButton>
        <GButton v-else variant="secondary" icon="stop" block @click="$emit('stop', project.id)">
          {{ t('project.inspector.stopProject') }}
        </GButton>
        <GButton variant="ghost" icon="edit" block @click="$emit('edit')">
          {{ t('project.inspector.editProject') }}
        </GButton>
        <GButton variant="ghost" icon="trash" block @click="$emit('delete')">
          {{ t('project.inspector.deleteProject') }}
        </GButton>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import GIcon from '@components/icons/GIcon.vue'
import GButton from '@components/base/GButton.vue'
import GStatusBadge from '@components/status/GStatusBadge.vue'
import ProjectTag from './ProjectTag.vue'
import type { Project } from '../types'
import { projectColorVars, formatBytes, formatDuration } from '../utils'

const props = defineProps<{ project: Project }>()
const { t } = useI18n()

defineEmits<{
  start: [id: string]
  stop: [id: string]
  edit: []
  delete: []
}>()

const colorVars = computed(() => projectColorVars(props.project.color))
const statusLabel = computed(() => t(`project.statusLabels.${props.project.status}`))

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

function dateLabel(value: string | number): string {
  const d = new Date(value)
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, '0')}-${String(d.getDate()).padStart(2, '0')} ${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}`
}
</script>

<style scoped>
.project-inspector {
  background: var(--bg-card);
  border: 1px solid var(--color-border);
  border-top: 3px solid var(--project-color);
  border-radius: var(--radius-card);
  overflow: hidden;
  display: flex;
  flex-direction: column;
  max-height: calc(100vh - 120px);
}

.project-inspector__header {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-3) var(--space-4);
  font-size: var(--text-sm);
  font-weight: var(--weight-semibold);
  color: var(--text-secondary);
  border-bottom: 1px solid var(--color-border-subtle);
}

.project-inspector__body {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-4);
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

/* ── Hero ── */
.project-inspector__hero {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding-bottom: var(--space-3);
  border-bottom: 1px solid var(--color-border-subtle);
}

.project-inspector__hero-icon {
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

.project-inspector__hero-text {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
  min-width: 0;
}

.project-inspector__hero-name {
  font-size: var(--text-md);
  font-weight: var(--weight-semibold);
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

/* ── 信息组 ── */
.project-inspector__group {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.project-inspector__group-title {
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
  color: var(--text-tertiary);
  text-transform: uppercase;
  letter-spacing: var(--tracking-wider);
}

.project-inspector__row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-2);
  font-size: var(--text-sm);
}

.project-inspector__label {
  color: var(--text-tertiary);
  flex-shrink: 0;
}

.project-inspector__value {
  color: var(--text-secondary);
  font-weight: var(--weight-medium);
  text-align: right;
  min-width: 0;
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
}

.project-inspector__value.mono {
  font-family: var(--font-mono);
  font-weight: var(--weight-regular);
  font-size: var(--text-xs);
}

.project-inspector__value :deep(.on) {
  color: var(--color-success);
}
.project-inspector__value :deep(.off) {
  color: var(--text-tertiary);
}

.project-inspector__tags {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-1);
}

.project-inspector__remark {
  font-size: var(--text-sm);
  color: var(--text-secondary);
  line-height: var(--leading-relaxed);
  padding: var(--space-2) var(--space-3);
  background: var(--bg-surface-hover);
  border-radius: var(--radius-md);
}

/* ── 操作 ── */
.project-inspector__actions {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
  padding-top: var(--space-3);
  border-top: 1px solid var(--color-border-subtle);
}
</style>
