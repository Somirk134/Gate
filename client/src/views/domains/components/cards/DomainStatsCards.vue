<template>
  <section class="domain-stats">
    <article
      v-for="card in cards"
      :key="card.key"
      class="domain-stat-card"
      :class="[`is-${card.tone}`, { 'domain-stat-card--loading': loading }]">
      <div class="domain-stat-card__body">
        <p>{{ card.label }}</p>
        <strong>{{ loading ? '—' : card.value }}</strong>
        <small v-if="card.helper">{{ card.helper }}</small>
      </div>
      <span class="domain-stat-card__icon">
        <GIcon :name="card.icon" :size="20" />
      </span>
    </article>
  </section>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import GIcon from '@components/icons/GIcon.vue'
import type { DomainStats } from '../../types'

const props = defineProps<{
  stats: DomainStats | null
  loading?: boolean
}>()

const { t } = useI18n()

const cards = computed(() => {
  const stats = props.stats
  const value = (count: number) => (props.loading ? '—' : String(count))

  return [
    {
      key: 'total',
      label: t('domains.stats.total'),
      value: value(stats?.total ?? 0),
      icon: 'globe',
      tone: 'primary',
    },
    {
      key: 'online',
      label: t('domains.stats.online'),
      value: value(stats?.online ?? 0),
      icon: 'check-circle',
      tone: 'success',
      helper: stats ? t('domains.stats.onlineHint', { total: stats.total }) : '',
    },
    {
      key: 'abnormal',
      label: t('domains.stats.abnormal'),
      value: value(stats?.abnormal ?? 0),
      icon: 'alert-circle',
      tone: (stats?.abnormal ?? 0) > 0 ? 'error' : 'neutral',
      helper: (stats?.abnormal ?? 0) > 0 ? t('domains.stats.abnormalHint') : '',
    },
    {
      key: 'unboundTunnel',
      label: t('domains.stats.unboundTunnel'),
      value: value(stats?.unboundTunnel ?? 0),
      icon: 'unlink',
      tone: (stats?.unboundTunnel ?? 0) > 0 ? 'warning' : 'neutral',
      helper: (stats?.unboundTunnel ?? 0) > 0 ? t('domains.stats.unboundHint') : '',
    },
  ]
})
</script>

<style scoped>
.domain-stats {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: var(--space-3);
}

.domain-stat-card {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: var(--space-3);
  padding: var(--space-4);
  border-radius: var(--radius-lg);
  border: 1px solid var(--border-subtle);
  background: var(--bg-surface);
  min-height: 96px;
}

.domain-stat-card__body {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
}

.domain-stat-card__body p {
  margin: 0;
  font-size: var(--text-xs);
  color: var(--text-secondary);
}

.domain-stat-card__body strong {
  font-size: var(--text-2xl);
  line-height: 1.1;
}

.domain-stat-card__body small {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  line-height: 1.4;
}

.domain-stat-card__icon {
  flex-shrink: 0;
  width: 36px;
  height: 36px;
  border-radius: var(--radius-md);
  display: grid;
  place-items: center;
  background: var(--bg-input);
  color: var(--text-secondary);
}

.domain-stat-card.is-success .domain-stat-card__icon { color: var(--color-success); }
.domain-stat-card.is-warning .domain-stat-card__icon { color: var(--color-warning); }
.domain-stat-card.is-error .domain-stat-card__icon { color: var(--color-danger); }
.domain-stat-card.is-primary .domain-stat-card__icon { color: var(--color-primary); }

.domain-stat-card--loading {
  opacity: 0.72;
}

@media (max-width: 960px) {
  .domain-stats {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}
</style>
