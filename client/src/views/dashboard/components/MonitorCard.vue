<!--
  MonitorCard — 资源监控
  ------------------------------------------------------------------
  显示 CPU / Memory / Traffic / Connection，使用环形进度。
  不使用大型图表。
-->
<template>
  <section class="dashboard-section">
    <GCard variant="plain" padding="none" class="monitor-card">
      <template #header>
        <GSectionHeader icon="activity">
          {{ title }}
        </GSectionHeader>
      </template>

      <div v-if="resource" class="monitor__grid">
        <div
          v-for="(item, i) in monitors"
          :key="item.key"
          class="monitor__item"
          :class="`stagger-${(i % 6) + 1}`">
          <GCircleProgress
            :value="item.value"
            :size="64"
            :stroke="5"
            :variant="item.variant"
            show-label />
          <div class="monitor__info">
            <span class="monitor__label">
              <GIcon :name="item.icon" :size="12" />
              {{ item.label }}
            </span>
            <span class="monitor__value">{{ item.value }}%</span>
          </div>
        </div>
      </div>

      <GEmptyState v-else title="暂无资源数据" />
    </GCard>
  </section>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import GCard from '@components/base/GCard.vue'
import GIcon from '@components/icons/GIcon.vue'
import GSectionHeader from '@components/layout/GSectionHeader.vue'
import GCircleProgress from '@components/feedback/GCircleProgress.vue'
import GEmptyState from '@components/feedback/GEmptyState.vue'
import type { DashboardResource } from '../types'

const props = withDefaults(
  defineProps<{
    resource: DashboardResource | null
    title?: string
  }>(),
  {
    title: '资源监控',
  },
)

function variantFor(v: number): 'success' | 'warning' | 'error' {
  if (v < 60) return 'success'
  if (v < 85) return 'warning'
  return 'error'
}

const monitors = computed(() => {
  const r = props.resource
  if (!r) return []
  return [
    { key: 'cpu', label: 'CPU', icon: 'cpu', value: r.cpu, variant: variantFor(r.cpu) },
    {
      key: 'memory',
      label: '内存',
      icon: 'memory-stick',
      value: r.memory,
      variant: variantFor(r.memory),
    },
    {
      key: 'traffic',
      label: '流量',
      icon: 'network',
      value: r.traffic,
      variant: variantFor(r.traffic),
    },
    {
      key: 'connection',
      label: '连接',
      icon: 'users',
      value: r.connection,
      variant: variantFor(r.connection),
    },
  ]
})
</script>

<style scoped>
.monitor-card {
  animation: g-slide-in-up var(--duration-base) var(--ease-out) both;
}
.monitor__grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--space-3);
  padding: var(--space-4);
}
.monitor__item {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-3);
  background: var(--bg-input);
  border-radius: var(--radius-md);
  animation: g-slide-in-up var(--duration-base) var(--ease-out) both;
}
.monitor__info {
  display: flex;
  flex-direction: column;
  gap: 2px;
  min-width: 0;
}
.monitor__label {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  text-transform: uppercase;
  letter-spacing: var(--tracking-wide);
}
.monitor__value {
  font-size: var(--text-lg);
  font-weight: var(--weight-semibold);
  color: var(--text-primary);
  font-variant-numeric: tabular-nums;
}
</style>
