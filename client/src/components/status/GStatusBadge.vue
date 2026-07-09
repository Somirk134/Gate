<!--
  GStatusBadge — 状态徽章（圆点 + 文字）
  ------------------------------------------------------------------
  用途：列表/卡片中展示“运行中 / 已停止 / 连接中 / 错误”等运行态。
  结合 GStatusDot 圆点与文字标签，统一所有状态展示。

  Props:
    status  online | offline | connecting | reconnecting | error | warning | updating | maintenance | starting
    label   文字（不传则用默认映射）
    size    sm | md

  内置 status → {label, pulse} 映射，业务可直接传 status。
-->
<template>
  <span
    class="g-status-badge"
    :class="[`g-status-badge--${size}`]"
  >
    <GStatusDot
      :status="dotStatus"
      :pulse="needsPulse"
      :size="size === 'sm' ? 'xs' : 'sm'"
    />
    <span class="g-status-badge__text">{{ displayLabel }}</span>
  </span>
</template>

<script setup lang="ts">
import { computed } from "vue"
import GStatusDot from "./GStatusDot.vue"

type Status =
  | "online"
  | "offline"
  | "connecting"
  | "reconnecting"
  | "error"
  | "warning"
  | "updating"
  | "maintenance"
  | "starting"

const props = withDefaults(
  defineProps<{
    status: Status
    label?: string
    size?: "sm" | "md"
  }>(),
  {
    size: "md",
  },
)

const defaultLabels: Record<Status, string> = {
  online: "在线",
  offline: "离线",
  connecting: "连接中",
  reconnecting: "重连中",
  error: "错误",
  warning: "警告",
  updating: "更新中",
  maintenance: "维护中",
  starting: "启动中",
}

const pulseStatuses: Status[] = ["connecting", "reconnecting", "updating", "starting"]

const displayLabel = computed(() => props.label ?? defaultLabels[props.status])
const needsPulse = computed(() => pulseStatuses.includes(props.status))
const dotStatus = computed(() => props.status as any)
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
