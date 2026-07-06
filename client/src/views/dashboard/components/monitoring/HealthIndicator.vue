<template>
  <div class="health-indicator" :class="`health-indicator--${status}`">
    <span class="health-indicator__icon" :title="statusLabel">
      <GIcon :name="icon" :size="16" />
    </span>
    <span class="health-indicator__main">
      <strong>{{ label }}</strong>
      <small>{{ message }}</small>
    </span>
    <span class="health-indicator__score">{{ Math.round(score) }}</span>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue"
import GIcon from "@components/icons/GIcon.vue"
import type { HealthStatus } from "@/monitoring/types"

const props = defineProps<{
  label: string
  status: HealthStatus
  score: number
  message: string
}>()

const icon = computed(() => {
  if (props.status === "healthy") return "check-circle"
  if (props.status === "warning") return "alert-triangle"
  if (props.status === "critical") return "alert-circle"
  return "wifi-off"
})

const statusLabel = computed(() => {
  if (props.status === "healthy") return "Healthy"
  if (props.status === "warning") return "Warning"
  if (props.status === "critical") return "Critical"
  return "Offline"
})
</script>

<style scoped>
.health-indicator {
  display: grid;
  grid-template-columns: 28px minmax(0, 1fr) 44px;
  align-items: center;
  gap: var(--space-3);
  min-height: 52px;
  padding: var(--space-3);
  border: 1px solid var(--color-border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
}

.health-indicator__icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: var(--radius-sm);
}

.health-indicator--healthy .health-indicator__icon { color: var(--color-success); background: var(--color-success-muted); }
.health-indicator--warning .health-indicator__icon { color: var(--color-warning); background: var(--color-warning-muted); }
.health-indicator--critical .health-indicator__icon { color: var(--color-error); background: var(--color-error-muted); }
.health-indicator--offline .health-indicator__icon { color: var(--text-tertiary); background: var(--bg-surface-hover); }

.health-indicator__main {
  display: flex;
  min-width: 0;
  flex-direction: column;
  gap: 2px;
}

.health-indicator__main strong {
  overflow: hidden;
  color: var(--text-primary);
  font-size: var(--text-sm);
  font-weight: var(--weight-semibold);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.health-indicator__main small {
  overflow: hidden;
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.health-indicator__score {
  color: var(--text-secondary);
  font-size: var(--text-md);
  font-weight: var(--weight-semibold);
  font-variant-numeric: tabular-nums;
  text-align: right;
}
</style>
