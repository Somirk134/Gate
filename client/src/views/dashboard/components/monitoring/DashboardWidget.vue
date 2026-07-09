<template>
  <section class="dashboard-widget">
    <header class="dashboard-widget__header">
      <div class="dashboard-widget__title">
        <GIcon
          v-if="icon"
          :name="icon"
          :size="16"
        />
        <span>{{ title }}</span>
      </div>
      <div
        v-if="$slots.action"
        class="dashboard-widget__action"
      >
        <slot name="action" />
      </div>
    </header>
    <div class="dashboard-widget__body">
      <slot />
    </div>
  </section>
</template>

<script setup lang="ts">
import GIcon from "@components/icons/GIcon.vue"

defineProps<{
  title: string
  icon?: string
}>()
</script>

<style scoped>
.dashboard-widget {
  display: flex;
  flex-direction: column;
  min-width: 0;
  background: var(--bg-card);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-md);
}

.dashboard-widget__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  min-height: 42px;
  padding: 0 var(--space-4);
  border-bottom: 1px solid var(--color-border-subtle);
}

.dashboard-widget__title {
  display: inline-flex;
  align-items: center;
  min-width: 0;
  gap: var(--space-2);
  color: var(--text-primary);
  font-size: var(--text-md);
  font-weight: var(--weight-semibold);
}

.dashboard-widget__title span {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.dashboard-widget__action {
  display: inline-flex;
  align-items: center;
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

.dashboard-widget__body {
  flex: 1;
  min-width: 0;
  padding: var(--space-4);
}
</style>
