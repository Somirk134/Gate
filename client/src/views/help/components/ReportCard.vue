<template>
  <button class="report-card" :class="`is-${tone}`" type="button" @click="$emit('action')">
    <span class="report-card__icon">
      <GIcon :name="icon" :size="18" />
    </span>
    <span class="report-card__copy">
      <strong>{{ title }}</strong>
      <small>{{ description }}</small>
    </span>
    <GIcon name="arrow-right" :size="15" class="report-card__arrow" />
  </button>
</template>

<script setup lang="ts">
import GIcon from '@components/icons/GIcon.vue'

withDefaults(
  defineProps<{
    title: string
    description: string
    icon: string
    tone?: 'default' | 'primary' | 'danger'
  }>(),
  {
    tone: 'default',
  },
)

defineEmits<{
  action: []
}>()
</script>

<style scoped>
.report-card {
  min-height: 86px;
  width: 100%;
  display: grid;
  grid-template-columns: 34px minmax(0, 1fr) 20px;
  align-items: center;
  gap: var(--space-3);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-input);
  color: var(--text-primary);
  padding: var(--space-3);
  text-align: left;
  cursor: pointer;
  transition:
    background-color var(--duration-fast) var(--ease-out),
    border-color var(--duration-fast) var(--ease-out),
    transform var(--duration-fast) var(--ease-out);
}

.report-card:hover {
  border-color: var(--color-primary);
  background: var(--bg-surface-hover);
  transform: translateY(-1px);
}

.report-card__icon {
  width: 34px;
  height: 34px;
  display: grid;
  place-items: center;
  border-radius: var(--radius-md);
  background: var(--bg-surface);
  color: var(--text-secondary);
}

.report-card__copy {
  min-width: 0;
  display: grid;
  gap: 3px;
}

.report-card strong {
  color: var(--text-primary);
  font-size: var(--text-sm);
  font-weight: var(--weight-semibold);
}

.report-card small {
  color: var(--text-secondary);
  line-height: var(--leading-normal);
  overflow-wrap: anywhere;
}

.report-card__arrow {
  color: var(--text-tertiary);
}

.report-card.is-primary .report-card__icon {
  background: var(--color-primary-muted);
  color: var(--color-primary);
}

.report-card.is-danger .report-card__icon {
  background: var(--color-error-muted);
  color: var(--color-error);
}
</style>
