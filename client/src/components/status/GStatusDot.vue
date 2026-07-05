<!--
  GStatusDot — 状态圆点
  ------------------------------------------------------------------
  用途：连接/运行状态的极简圆点指示。支持脉冲（连接中）。
  Props:
    status  online | offline | connecting | error | warning | idle
    pulse   是否脉冲动画
    size    xs(6) | sm(8) | md(10)
-->
<template>
  <span class="g-status-dot" :class="[`g-status-dot--${status}`, `g-status-dot--${size}`, { 'g-status-dot--pulse': pulse }]">
    <span v-if="ping" class="g-status-dot__ping" />
  </span>
</template>

<script setup lang="ts">
withDefaults(
  defineProps<{
    status?: "online" | "offline" | "connecting" | "starting" | "error" | "warning" | "idle"
    pulse?: boolean
    ping?: boolean
    size?: "xs" | "sm" | "md"
  }>(),
  {
    status: "offline",
    pulse: false,
    ping: false,
    size: "sm",
  },
)
</script>

<style scoped>
.g-status-dot {
  position: relative;
  display: inline-block;
  border-radius: var(--radius-full);
  flex-shrink: 0;
}
.g-status-dot--xs { width: 6px; height: 6px; }
.g-status-dot--sm { width: 8px; height: 8px; }
.g-status-dot--md { width: 10px; height: 10px; }

.g-status-dot--online      { background: var(--status-online); }
.g-status-dot--offline     { background: var(--status-offline); }
.g-status-dot--connecting  { background: var(--status-connecting); }
.g-status-dot--starting    { background: var(--status-starting); }
.g-status-dot--error       { background: var(--status-error); }
.g-status-dot--warning     { background: var(--status-warning); }
.g-status-dot--idle        { background: var(--text-tertiary); }

.g-status-dot--pulse {
  animation: g-pulse 1.5s var(--ease-in-out) infinite;
}

.g-status-dot__ping {
  position: absolute;
  inset: 0;
  border-radius: var(--radius-full);
  background: inherit;
  animation: g-ping 1.5s var(--ease-out) infinite;
}
</style>
