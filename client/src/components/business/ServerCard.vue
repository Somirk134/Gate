<!--
  ServerCard — 服务器卡片（业务组件模板）
  ------------------------------------------------------------------
  用途：展示单台服务器（地址/延迟/负载/隧道数/状态）。
  属业务组件：基于 DS 组合，无逻辑。

  Props:
    name        服务器名
    host        主机地址
    latency     延迟(ms)
    load        负载 0-100
    tunnelCount 隧道数
    status      状态

  Events: click / action(key)

  复用：GCard / GIcon / GStatusBadge / GIconButton / GCircleProgress
-->
<template>
  <GCard variant="plain" padding="md">
    <div class="server-card">
      <div class="server-card__head">
        <span class="server-card__icon">
          <GIcon name="server" :size="18" />
        </span>
        <div class="server-card__title-wrap">
          <span class="server-card__name">{{ name }}</span>
          <span class="server-card__host">{{ host }}</span>
        </div>
        <GStatusBadge :status="status" size="sm" />
        <GIconButton name="more-horizontal" size="sm" @click="emit('action', 'menu')" />
      </div>

      <div class="server-card__metrics">
        <div class="server-card__metric">
          <span class="server-card__metric-label">延迟</span>
          <span class="server-card__metric-value" :class="latencyClass">{{ latency }}ms</span>
        </div>
        <div class="server-card__metric">
          <span class="server-card__metric-label">负载</span>
          <GCircleProgress :value="load" :size="36" :stroke="3" :variant="loadVariant" />
        </div>
        <div class="server-card__metric">
          <span class="server-card__metric-label">隧道</span>
          <span class="server-card__metric-value">{{ tunnelCount }}</span>
        </div>
      </div>
    </div>
  </GCard>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import GCard from '@components/base/GCard.vue'
import GIcon from '@components/icons/GIcon.vue'
import GIconButton from '@components/base/GIconButton.vue'
import GStatusBadge from '@components/status/GStatusBadge.vue'
import GCircleProgress from '@components/feedback/GCircleProgress.vue'

const props = withDefaults(
  defineProps<{
    name: string
    host: string
    latency: number
    load: number
    tunnelCount: number
    status: 'online' | 'offline' | 'connecting' | 'error' | 'warning' | 'maintenance'
  }>(),
  {},
)

const emit = defineEmits<{ click: []; action: [key: string] }>()

const latencyClass = computed(() => {
  if (props.latency < 100) return 'server-card__metric-value--good'
  if (props.latency < 300) return 'server-card__metric-value--warn'
  return 'server-card__metric-value--bad'
})

const loadVariant = computed<'success' | 'warning' | 'error'>(() => {
  if (props.load < 60) return 'success'
  if (props.load < 85) return 'warning'
  return 'error'
})
</script>

<style scoped>
.server-card {
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}
.server-card__head {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}
.server-card__icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border-radius: var(--radius-md);
  background: var(--bg-surface-hover);
  color: var(--text-secondary);
  flex-shrink: 0;
}
.server-card__title-wrap {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}
.server-card__name {
  font-size: var(--text-md);
  font-weight: var(--weight-semibold);
  color: var(--text-primary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.server-card__host {
  font-size: var(--text-xs);
  font-family: var(--font-mono);
  color: var(--text-tertiary);
}
.server-card__metrics {
  display: grid;
  grid-template-columns: 1fr 1fr 1fr;
  gap: var(--space-3);
  align-items: center;
}
.server-card__metric {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-1);
}
.server-card__metric-label {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  text-transform: uppercase;
  letter-spacing: var(--tracking-wider);
}
.server-card__metric-value {
  font-size: var(--text-md);
  font-weight: var(--weight-semibold);
  color: var(--text-primary);
  font-variant-numeric: tabular-nums;
}
.server-card__metric-value--good {
  color: var(--color-success);
}
.server-card__metric-value--warn {
  color: var(--color-warning);
}
.server-card__metric-value--bad {
  color: var(--color-error);
}
</style>
