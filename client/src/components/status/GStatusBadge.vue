<!--
  GStatusBadge - 状态徽章。
  默认状态文案统一走 i18n，避免语言切换后仍显示旧语言。
-->
<template>
  <span class="g-status-badge" :class="[`g-status-badge--${size}`]">
    <GStatusDot :status="dotStatus" :pulse="needsPulse" :size="size === 'sm' ? 'xs' : 'sm'" />
    <span class="g-status-badge__text">{{ displayLabel }}</span>
  </span>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import GStatusDot from './GStatusDot.vue'

type Status =
  | 'online'
  | 'offline'
  | 'connecting'
  | 'reconnecting'
  | 'error'
  | 'warning'
  | 'updating'
  | 'maintenance'
  | 'starting'

const props = withDefaults(
  defineProps<{
    status: Status
    label?: string
    size?: 'sm' | 'md'
  }>(),
  {
    size: 'md',
  },
)

const { t } = useI18n()

const pulseStatuses: Status[] = ['connecting', 'reconnecting', 'updating', 'starting']
const dotStatusMap: Record<Status, 'online' | 'offline' | 'connecting' | 'starting' | 'error' | 'warning'> = {
  online: 'online',
  offline: 'offline',
  connecting: 'connecting',
  reconnecting: 'connecting',
  error: 'error',
  warning: 'warning',
  updating: 'connecting',
  maintenance: 'warning',
  starting: 'starting',
}

const displayLabel = computed(() => props.label ?? t(`statusBadge.${props.status}`))
const needsPulse = computed(() => pulseStatuses.includes(props.status))
const dotStatus = computed(() => dotStatusMap[props.status])
</script>

<style scoped>
.g-status-badge {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  vertical-align: middle;
}
.g-status-badge--sm .g-status-badge__text {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
}
.g-status-badge--md .g-status-badge__text {
  font-size: var(--text-sm);
  color: var(--text-secondary);
}
</style>
