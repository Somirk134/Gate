<template>
  <button
    class="nav-item"
    :class="{ active, collapsed }"
    :title="collapsed ? item.label : undefined"
    @click="$emit('click')"
  >
    <span class="nav-icon" v-html="getIcon(item.icon)"></span>
    <span v-show="!collapsed" class="nav-label">{{ item.label }}</span>
    <span v-if="item.badge && !collapsed" class="nav-badge">{{ item.badge }}</span>
  </button>
</template>

<script setup lang="ts">
defineProps<{
  item: { path: string; label: string; icon: string; badge?: number }
  collapsed: boolean
  active: boolean
}>()

defineEmits<{
  click: []
}>()

function getIcon(name: string): string {
  const icons: Record<string, string> = {
    dashboard: '<svg viewBox="0 0 18 18" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="1" y="1" width="6" height="7" rx="1"/><rect x="11" y="1" width="6" height="5" rx="1"/><rect x="1" y="12" width="6" height="5" rx="1"/><rect x="11" y="10" width="6" height="7" rx="1"/></svg>',
    projects: '<svg viewBox="0 0 18 18" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M2 3h6l2 3h6v8a1 1 0 0 1-1 1H3a1 1 0 0 1-1-1V4a1 1 0 0 1 1-1z"/></svg>',
    servers: '<svg viewBox="0 0 18 18" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="2" y="2" width="14" height="5" rx="1"/><rect x="2" y="9" width="14" height="5" rx="1"/><circle cx="5" cy="4.5" r="0.8" fill="currentColor"/><circle cx="5" cy="11.5" r="0.8" fill="currentColor"/></svg>',
    logs: '<svg viewBox="0 0 18 18" fill="none" stroke="currentColor" stroke-width="1.5"><polyline points="4 6 8 10 12 6"/><line x1="8" y1="10" x2="8" y2="14"/><rect x="2" y="2" width="14" height="14" rx="2"/></svg>',
    settings: '<svg viewBox="0 0 18 18" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="9" cy="9" r="2.5"/><path d="M9 1.5v2M9 14.5v2M14.3 3.7l-1.4 1.4M5.1 12.9l-1.4 1.4M16.5 9h-2M3.5 9h-2M14.3 14.3l-1.4-1.4M5.1 5.1L3.7 3.7"/></svg>',
    about: '<svg viewBox="0 0 18 18" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="9" cy="9" r="7"/><line x1="9" y1="5" x2="9" y2="5.01"/><line x1="9" y1="8" x2="9" y2="13"/></svg>',
    connection: '<svg viewBox="0 0 18 18" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="9" cy="9" r="7"/><polyline points="5 10 8 13 13 8"/></svg>',
    tunnel: '<svg viewBox="0 0 18 18" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M3 8h12M3 13h12M3 3h12"/></svg>',
  }
  return icons[name] || icons.dashboard
}
</script>

<style scoped>
.nav-item {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  width: 100%;
  padding: var(--space-2) var(--space-3);
  border: none;
  background: transparent;
  color: var(--text-secondary);
  font-family: var(--font-ui);
  font-size: var(--text-base);
  font-weight: 400;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--duration-micro) var(--ease-out);
  text-align: left;
  position: relative;
}

.nav-item:hover {
  background: rgba(255, 255, 255, 0.04);
  color: var(--text-primary);
}

.nav-item.active {
  background: var(--accent-primary-muted);
  color: var(--accent-primary);
  font-weight: 500;
}

.nav-item.active::before {
  content: "";
  position: absolute;
  left: 0;
  top: 4px;
  bottom: 4px;
  width: 3px;
  background: var(--accent-primary);
  border-radius: 0 var(--radius-full) var(--radius-full) 0;
}

.nav-item.collapsed {
  justify-content: center;
  padding: var(--space-2);
}

.nav-icon {
  width: 18px;
  height: 18px;
  flex-shrink: 0;
  display: flex;
  align-items: center;
  justify-content: center;
}

.nav-icon :deep(svg) {
  width: 100%;
  height: 100%;
}

.nav-label {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.nav-badge {
  margin-left: auto;
  padding: 0 6px;
  height: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--accent-primary);
  color: white;
  font-size: var(--text-xs);
  font-weight: 600;
  border-radius: var(--radius-full);
  min-width: 18px;
}
</style>
