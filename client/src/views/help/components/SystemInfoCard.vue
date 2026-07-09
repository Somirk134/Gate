<template>
  <section class="system-info-card">
    <header>
      <div>
        <strong>{{ title }}</strong>
        <p v-if="description">{{ description }}</p>
      </div>
      <slot name="actions" />
    </header>

    <dl>
      <div v-for="row in rows" :key="row.label">
        <dt>{{ row.label }}</dt>
        <dd :class="{ muted: row.muted }">{{ row.value }}</dd>
      </div>
    </dl>
  </section>
</template>

<script setup lang="ts">
export interface SystemInfoRow {
  label: string
  value: string
  muted?: boolean
}

defineProps<{
  title: string
  description?: string
  rows: SystemInfoRow[]
}>()
</script>

<style scoped>
.system-info-card {
  display: grid;
  gap: var(--space-4);
  padding: var(--space-4);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
}

.system-info-card header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-3);
}

.system-info-card strong {
  color: var(--text-primary);
  font-size: var(--text-lg);
  font-weight: var(--weight-semibold);
}

.system-info-card p {
  margin-top: 2px;
  color: var(--text-secondary);
  line-height: var(--leading-normal);
}

.system-info-card dl {
  display: grid;
  gap: var(--space-2);
}

.system-info-card dl div {
  display: grid;
  grid-template-columns: 112px minmax(0, 1fr);
  gap: var(--space-3);
  min-height: 28px;
  align-items: baseline;
}

.system-info-card dt {
  color: var(--text-tertiary);
}

.system-info-card dd {
  color: var(--text-primary);
  overflow-wrap: anywhere;
}

.system-info-card dd.muted {
  color: var(--text-tertiary);
}

@media (max-width: 720px) {
  .system-info-card dl div {
    grid-template-columns: 1fr;
    gap: 2px;
  }
}
</style>
