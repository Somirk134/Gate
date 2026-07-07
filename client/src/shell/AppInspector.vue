<template>
    <aside class="app-inspector" :style="{ minWidth: '240px', maxWidth: layout.inspectorWidth + 'px' }">
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
                @click="activeTab = tab.id"
            >
                <GIcon v-if="tab.icon" :name="tab.icon" :size="14" />
                <span>{{ tab.label }}</span>
            </div>
        </div>

        <div class="inspector-content">
            <div class="inspector-panel" v-show="activeTab === 'details'">
                <div class="inspector-placeholder">
                    <p class="placeholder-text">暂无数据</p>
                    <p class="placeholder-subtext">当前页面未提供详情数据。</p>
                </div>
            </div>
            <div class="inspector-panel" v-show="activeTab === 'logs'">
                <div class="inspector-placeholder">
                    <p class="placeholder-text">暂无日志</p>
                    <p class="placeholder-subtext">当前上下文没有可显示的实时日志。</p>
                </div>
            </div>
            <div class="inspector-panel" v-show="activeTab === 'stats'">
                <div class="inspector-placeholder">
                    <p class="placeholder-text">暂无数据</p>
                    <p class="placeholder-subtext">该统计面板暂未接入真实 Runtime 指标。</p>
                </div>
            </div>
            <div class="inspector-panel" v-show="activeTab === 'properties'">
                <div class="inspector-placeholder">
                    <p class="placeholder-text">暂无属性</p>
                    <p class="placeholder-subtext">请选择已接入的数据对象后查看属性。</p>
                </div>
            </div>
        </div>

        <!-- Resize Handle (预留) -->
        <div class="inspector-resize-handle" @mousedown="startResize"></div>
    </aside>
</template>

<script setup lang="ts">
import { ref, computed } from "vue"
import { useLayoutStore } from "@stores"
import GIcon from "@components/icons/GIcon.vue"

const layout = useLayoutStore()

const activeTab = ref('details')

const tabs = [
    { id: 'details', label: 'Details', icon: 'file-text' },
    { id: 'logs', label: 'Logs', icon: 'scroll-text' },
    { id: 'stats', label: 'Stats', icon: 'chart-bar' },
    { id: 'properties', label: 'Properties', icon: 'list' },
]

const activeTabLabel = computed(() => {
    return tabs.find(t => t.id === activeTab.value)?.label || 'Details'
})

function cycleTab() {
    const idx = tabs.findIndex(t => t.id === activeTab.value)
    activeTab.value = tabs[(idx + 1) % tabs.length].id
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
