<template>
  <aside
    class="app-inspector"
    :style="{ minWidth: '240px', maxWidth: layout.inspectorWidth + 'px' }">
    <div class="inspector-header">
      <span class="inspector-title">{{ activeTabLabel }}</span>
      <div class="inspector-actions">
        <button class="inspector-btn" @click="cycleTab">
          <GIcon name="layers" :size="14" />
        </button>
        <button class="inspector-btn" @click="layout.closeInspector">
          <GIcon name="x" :size="14" />
        </button>
      </div>
    </div>

    <div class="inspector-tabs">
      <div
        v-for="tab in tabs"
        :key="tab.id"
        class="inspector-tab"
        :class="{ active: activeTab === tab.id }"
        @click="activeTab = tab.id">
        <GIcon v-if="tab.icon" :name="tab.icon" :size="14" />
        <span>{{ tab.label }}</span>
      </div>
    </div>

    <div class="inspector-content">
      <div v-show="activeTab === 'details'" class="inspector-panel">
        <div class="inspector-placeholder">
          <p class="placeholder-text">{{ t('inspector.empty.detailsTitle') }}</p>
          <p class="placeholder-subtext">{{ t('inspector.empty.detailsDesc') }}</p>
        </div>
      </div>
      <div v-show="activeTab === 'logs'" class="inspector-panel">
        <div class="inspector-placeholder">
          <p class="placeholder-text">{{ t('inspector.empty.logsTitle') }}</p>
          <p class="placeholder-subtext">{{ t('inspector.empty.logsDesc') }}</p>
        </div>
      </div>
      <div v-show="activeTab === 'stats'" class="inspector-panel">
        <div class="inspector-placeholder">
          <p class="placeholder-text">{{ t('inspector.empty.statsTitle') }}</p>
          <p class="placeholder-subtext">{{ t('inspector.empty.statsDesc') }}</p>
        </div>
      </div>
      <div v-show="activeTab === 'properties'" class="inspector-panel">
        <div class="inspector-placeholder">
          <p class="placeholder-text">{{ t('inspector.empty.propertiesTitle') }}</p>
          <p class="placeholder-subtext">{{ t('inspector.empty.propertiesDesc') }}</p>
        </div>
      </div>
    </div>

    <!-- Resize Handle (预留) -->
    <div class="inspector-resize-handle" @mousedown="startResize" />
  </aside>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useLayoutStore } from '@stores'
import GIcon from '@components/icons/GIcon.vue'

const layout = useLayoutStore()
const { t } = useI18n()

const activeTab = ref('details')

const tabs = computed(() => [
  { id: 'details', label: t('inspector.tabs.details'), icon: 'file-text' },
  { id: 'logs', label: t('inspector.tabs.logs'), icon: 'scroll-text' },
  { id: 'stats', label: t('inspector.tabs.stats'), icon: 'chart-bar' },
  { id: 'properties', label: t('inspector.tabs.properties'), icon: 'list' },
])

const activeTabLabel = computed(() => {
  return tabs.value.find((tab) => tab.id === activeTab.value)?.label || t('inspector.tabs.details')
})

function cycleTab() {
  const idx = tabs.value.findIndex((tab) => tab.id === activeTab.value)
  activeTab.value = tabs.value[(idx + 1) % tabs.value.length].id
}

function startResize(e: MouseEvent) {
  const startX = e.clientX
  const startWidth = layout.inspectorWidth

  function onMouseMove(ev: MouseEvent) {
    const delta = startX - ev.clientX
    layout.setInspectorWidth(startWidth + delta)
  }

  function onMouseUp() {
    document.removeEventListener('mousemove', onMouseMove)
    document.removeEventListener('mouseup', onMouseUp)
  }

  document.addEventListener('mousemove', onMouseMove)
  document.addEventListener('mouseup', onMouseUp)
}
</script>

<style scoped>
.app-inspector {
  display: flex;
  flex-direction: column;
  background: var(--bg-sidebar);
  border-left: 1px solid var(--border-subtle);
  overflow: hidden;
  position: relative;
  min-width: 240px;
  max-width: 480px;
  transition: width var(--duration-standard) var(--ease-out);
}

.inspector-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: var(--toolbar-height);
  padding: 0 var(--space-3);
  border-bottom: 1px solid var(--border-subtle);
  flex-shrink: 0;
}

.inspector-title {
  font-size: var(--text-sm);
  font-weight: 500;
  color: var(--text-secondary);
}

.inspector-actions {
  display: flex;
  align-items: center;
  gap: var(--space-1);
}

.inspector-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: all var(--duration-micro);
}

.inspector-btn:hover {
  background: var(--bg-surface-hover);
  color: var(--text-primary);
}

/* ── Tabs ── */
.inspector-tabs {
  display: flex;
  gap: var(--space-1);
  padding: var(--space-2) var(--space-3);
  border-bottom: 1px solid var(--border-subtle);
  flex-shrink: 0;
  overflow-x: auto;
}

.inspector-tab {
  display: flex;
  align-items: center;
  gap: var(--space-1);
  padding: var(--space-1) var(--space-2);
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  border-radius: var(--radius-sm);
  cursor: pointer;
  white-space: nowrap;
  transition: all var(--duration-micro);
}

.inspector-tab:hover {
  color: var(--text-secondary);
  background: var(--bg-surface-hover);
}

.inspector-tab.active {
  color: var(--color-primary);
  background: var(--color-primary-muted);
  font-weight: 500;
}

/* ── Content ── */
.inspector-content {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-4);
}

.inspector-panel {
  height: 100%;
}

.inspector-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  gap: var(--space-2);
  text-align: center;
}

.placeholder-text {
  font-size: var(--text-sm);
  font-weight: 500;
  color: var(--text-secondary);
}

.placeholder-subtext {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  max-width: 200px;
}

/* ── Resize Handle ── */
.inspector-resize-handle {
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 4px;
  cursor: col-resize;
  transition: background var(--duration-micro);
}

.inspector-resize-handle:hover {
  background: var(--color-primary);
}
</style>
