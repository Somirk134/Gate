<template>
  <article
    class="statistics-card"
    :class="`statistics-card--${tone}`"
  >
    <div
      class="statistics-card__icon"
      :title="label"
    >
      <GIcon
        :name="icon"
        :size="18"
      />
    </div>
    <div class="statistics-card__content">
      <span class="statistics-card__label">{{ label }}</span>
      <strong class="statistics-card__value">{{ value }}</strong>
      <span
        v-if="meta"
        class="statistics-card__meta"
      >{{ meta }}</span>
    </div>
  </article>
</template>

<script setup lang="ts">
import GIcon from "@components/icons/GIcon.vue"

withDefaults(
  defineProps<{
    label: string
    value: string
    icon: string
    meta?: string
    tone?: "neutral" | "primary" | "success" | "warning" | "danger" | "info"
  }>(),
  {
    tone: "neutral",
  },
)
</script>

<style scoped>
.statistics-card {
  display: grid;
  grid-template-columns: 36px minmax(0, 1fr);
  align-items: center;
  gap: var(--space-3);
  min-height: 92px;
  padding: var(--space-4);
  background: var(--bg-card);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
}

.statistics-card__icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  background: var(--bg-surface-hover);
}

.statistics-card--primary .statistics-card__icon { color: var(--color-primary); background: var(--color-primary-muted); }
.statistics-card--success .statistics-card__icon { color: var(--color-success); background: var(--color-success-muted); }
.statistics-card--warning .statistics-card__icon { color: var(--color-warning); background: var(--color-warning-muted); }
.statistics-card--danger .statistics-card__icon { color: var(--color-error); background: var(--color-error-muted); }
.statistics-card--info .statistics-card__icon { color: var(--color-info); background: var(--color-info-muted); }

.statistics-card__content {
  display: flex;
  min-width: 0;
  flex-direction: column;
  gap: 2px;
}

.statistics-card__label,
.statistics-card__meta {
  overflow: hidden;
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.statistics-card__value {
  overflow: hidden;
  color: var(--text-primary);
  font-size: var(--text-xl);
  font-weight: var(--weight-semibold);
  font-variant-numeric: tabular-nums;
  line-height: var(--leading-tight);
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
