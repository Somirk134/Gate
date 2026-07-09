<template>
  <article class="status-card" :class="`is-${status}`">
    <div class="status-card__icon">
      <GIcon :name="loading ? 'loader' : icon" :size="18" :spin="loading" />
    </div>
    <div class="status-card__body">
      <div class="status-card__topline">
        <strong>{{ title }}</strong>
        <span>{{ statusLabel }}</span>
      </div>
      <p>{{ value }}</p>
      <small>{{ detail }}</small>
    </div>
  </article>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import GIcon from '@components/icons/GIcon.vue'

type SupportStatus = 'ok' | 'warning' | 'error' | 'unknown'

const props = withDefaults(
  defineProps<{
    title: string
    value: string
    detail: string
    status: SupportStatus
    icon: string
    loading?: boolean
  }>(),
  {
    loading: false,
  },
)

const statusLabel = computed(() => {
  if (props.loading) return '检测中'
  if (props.status === 'ok') return '正常'
  if (props.status === 'warning') return '注意'
  if (props.status === 'error') return '异常'
  return '未知'
})
</script>

<style scoped>
.status-card {
  min-height: 112px;
  display: grid;
  grid-template-columns: 34px minmax(0, 1fr);
  gap: var(--space-3);
  padding: var(--space-4);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: linear-gradient(180deg, rgba(255, 255, 255, 0.035), transparent), var(--bg-surface);
}

.status-card__icon {
  width: 34px;
  height: 34px;
  display: grid;
  place-items: center;
  border-radius: var(--radius-md);
  background: var(--bg-input);
  color: var(--text-secondary);
}

.status-card__body {
  min-width: 0;
  display: grid;
  gap: var(--space-1);
}

.status-card__topline {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-2);
}

.status-card strong {
  color: var(--text-primary);
  font-size: var(--text-sm);
  font-weight: var(--weight-semibold);
}

.status-card__topline span {
  min-height: 22px;
  display: inline-flex;
  align-items: center;
  border-radius: var(--radius-sm);
  padding: 0 var(--space-2);
  background: var(--bg-input);
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  white-space: nowrap;
}

.status-card p {
  color: var(--text-primary);
  font-size: var(--text-lg);
  font-weight: var(--weight-semibold);
  overflow-wrap: anywhere;
}

.status-card small {
  color: var(--text-secondary);
  line-height: var(--leading-normal);
  overflow-wrap: anywhere;
}

.status-card.is-ok .status-card__icon,
.status-card.is-ok .status-card__topline span {
  background: var(--color-success-muted);
  color: var(--color-success);
}

.status-card.is-warning .status-card__icon,
.status-card.is-warning .status-card__topline span {
  background: var(--color-warning-muted);
  color: var(--color-warning);
}

.status-card.is-error .status-card__icon,
.status-card.is-error .status-card__topline span {
  background: var(--color-error-muted);
  color: var(--color-error);
}
</style>
