<template>
  <article class="diagnostic-card" :class="`is-${status}`">
    <div class="diagnostic-card__status">
      <GIcon :name="statusIcon" :size="17" />
      <span>{{ statusLabel }}</span>
    </div>
    <div class="diagnostic-card__content">
      <div>
        <strong>{{ title }}</strong>
        <small v-if="meta">{{ meta }}</small>
      </div>
      <p>{{ description }}</p>
      <dl>
        <div>
          <dt>结果</dt>
          <dd>{{ reason }}</dd>
        </div>
        <div>
          <dt>建议</dt>
          <dd>{{ solution }}</dd>
        </div>
      </dl>
    </div>
  </article>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import GIcon from '@components/icons/GIcon.vue'

type DiagnosticStatus = 'ok' | 'warning' | 'error'

const props = defineProps<{
  title: string
  description: string
  status: DiagnosticStatus
  reason: string
  solution: string
  meta?: string
}>()

const statusIcon = computed(() => {
  if (props.status === 'ok') return 'check-circle'
  if (props.status === 'warning') return 'alert-triangle'
  return 'alert-circle'
})

const statusLabel = computed(() => {
  if (props.status === 'ok') return '通过'
  if (props.status === 'warning') return '警告'
  return '失败'
})
</script>

<style scoped>
.diagnostic-card {
  display: grid;
  grid-template-columns: 88px minmax(0, 1fr);
  gap: var(--space-4);
  padding: var(--space-4);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
}

.diagnostic-card__status {
  display: flex;
  align-items: flex-start;
  gap: var(--space-2);
  color: var(--text-tertiary);
  font-size: var(--text-sm);
  font-weight: var(--weight-semibold);
}

.diagnostic-card__content {
  min-width: 0;
  display: grid;
  gap: var(--space-2);
}

.diagnostic-card__content > div {
  display: flex;
  align-items: baseline;
  justify-content: space-between;
  gap: var(--space-3);
}

.diagnostic-card strong {
  color: var(--text-primary);
  font-size: var(--text-md);
}

.diagnostic-card small {
  color: var(--text-tertiary);
  font-family: var(--font-mono);
  white-space: nowrap;
}

.diagnostic-card p {
  color: var(--text-secondary);
  line-height: var(--leading-normal);
}

.diagnostic-card dl {
  display: grid;
  gap: var(--space-2);
}

.diagnostic-card dl div {
  display: grid;
  grid-template-columns: 48px minmax(0, 1fr);
  gap: var(--space-3);
}

.diagnostic-card dt {
  color: var(--text-tertiary);
}

.diagnostic-card dd {
  color: var(--text-primary);
  overflow-wrap: anywhere;
}

.diagnostic-card.is-ok .diagnostic-card__status {
  color: var(--color-success);
}

.diagnostic-card.is-warning .diagnostic-card__status {
  color: var(--color-warning);
}

.diagnostic-card.is-error .diagnostic-card__status {
  color: var(--color-error);
}

@media (max-width: 720px) {
  .diagnostic-card {
    grid-template-columns: 1fr;
  }

  .diagnostic-card__content > div {
    align-items: flex-start;
    flex-direction: column;
    gap: var(--space-1);
  }
}
</style>
