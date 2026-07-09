<!--
  GStatCard — 统计卡片
  ------------------------------------------------------------------
  用途：仪表盘/概览页的数字统计卡片（总数/在线/流量等）。
  基于 GCard，统一图标、数值、标签、趋势布局。

  Props:
    label   标签
    value   数值
    icon    图标名
    variant 颜色语义（图标着色）
    trend   趋势：up/down + 文字

  属业务卡片基类，可被 StatisticsCard 等复用。
-->
<template>
  <GCard variant="plain" padding="md">
    <div class="g-stat-card">
      <div class="g-stat-card__head">
        <span class="g-stat-card__icon" :class="`g-stat-card__icon--${variant}`">
          <GIcon v-if="icon" :name="icon" :size="18" />
        </span>
        <span v-if="trend" class="g-stat-card__trend" :class="`g-stat-card__trend--${trendDir}`">
          <GIcon :name="trendDir === 'up' ? 'trending-up' : 'trending-down'" :size="12" />
          {{ trend }}
        </span>
      </div>
      <div class="g-stat-card__value">
        <slot>{{ value }}</slot>
      </div>
      <div class="g-stat-card__label">
        {{ label }}
      </div>
    </div>
  </GCard>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import GCard from '@components/base/GCard.vue'
import GIcon from '@components/icons/GIcon.vue'

const props = withDefaults(
  defineProps<{
    label: string
    value?: string | number
    icon?: string
    variant?: 'neutral' | 'primary' | 'success' | 'warning' | 'error' | 'info'
    trend?: string
  }>(),
  {
    variant: 'neutral',
  },
)

const trendDir = computed(() => (props.trend?.startsWith('-') ? 'down' : 'up'))
</script>

<style scoped>
.g-stat-card {
  display: flex;
  flex-direction: column;
  gap: var(--space-1);
}
.g-stat-card__head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: var(--space-2);
}
.g-stat-card__icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: var(--radius-md);
  background: var(--bg-surface-hover);
  color: var(--text-tertiary);
}
.g-stat-card__icon--primary {
  background: var(--color-primary-muted);
  color: var(--color-primary);
}
.g-stat-card__icon--success {
  background: var(--color-success-muted);
  color: var(--color-success);
}
.g-stat-card__icon--warning {
  background: var(--color-warning-muted);
  color: var(--color-warning);
}
.g-stat-card__icon--error {
  background: var(--color-error-muted);
  color: var(--color-error);
}
.g-stat-card__icon--info {
  background: var(--color-info-muted);
  color: var(--color-info);
}

.g-stat-card__value {
  font-size: var(--text-2xl);
  font-weight: var(--weight-semibold);
  color: var(--text-primary);
  font-variant-numeric: tabular-nums;
  line-height: var(--leading-tight);
}
.g-stat-card__label {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  text-transform: uppercase;
  letter-spacing: var(--tracking-wider);
}
.g-stat-card__trend {
  display: inline-flex;
  align-items: center;
  gap: 2px;
  font-size: var(--text-xs);
  font-weight: var(--weight-medium);
}
.g-stat-card__trend--up {
  color: var(--color-success);
}
.g-stat-card__trend--down {
  color: var(--color-error);
}
</style>
