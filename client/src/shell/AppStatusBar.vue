<template>
    <footer class="app-statusbar">
        <div class="statusbar-left">
            <div class="status-item" :class="statusClass">
                <span class="status-dot"></span>
                <span class="status-label">{{ statusLabel }}</span>
            </div>
            <div class="status-item" v-if="connectionStatus">
                <GIcon name="wifi" :size="12" />
                <span class="status-label">Connected</span>
            </div>
        </div>

        <div class="statusbar-center">
            <span class="status-version">Gate v{{ version }}</span>
        </div>

        <div class="statusbar-right">
            <span class="status-item">CPU: --</span>
            <span class="status-item">Mem: --</span>
            <span class="status-item">Traffic: --</span>
            <span class="status-item status-time">{{ currentTime }}</span>
        </div>
    </footer>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue"
import GIcon from "@components/icons/GIcon.vue"

const version = "0.1.0"
const statusLabel = ref('Ready')
const statusClass = ref('online')
const connectionStatus = ref(true)
const currentTime = ref('')

let timer: number

function updateTime() {
    const now = new Date()
    currentTime.value = now.toLocaleTimeString('zh-CN', {
        hour: '2-digit',
        minute: '2-digit',
        second: '2-digit',
    })
}

onMounted(() => {
    updateTime()
    timer = window.setInterval(updateTime, 1000)
})

onUnmounted(() => {
    clearInterval(timer)
})
</script>

<style scoped>
.app-statusbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    height: var(--statusbar-height);
    padding: 0 var(--space-4);
    background: var(--bg-statusbar);
    border-top: 1px solid var(--border-subtle);
    font-size: var(--text-xs);
    color: var(--text-muted);
    flex-shrink: 0;
    gap: var(--space-4);
}

.statusbar-left,
.statusbar-right {
    display: flex;
    align-items: center;
    gap: var(--space-4);
}

.statusbar-center {
    display: flex;
    align-items: center;
}

.status-item {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    white-space: nowrap;
}

.status-dot {
    width: 6px;
    height: 6px;
    border-radius: var(--radius-full);
    background: var(--status-online);
    flex-shrink: 0;
}

.status-item.online .status-dot {
    background: var(--status-online);
}

.status-item.offline .status-dot {
    background: var(--status-offline);
}

.status-item.warning .status-dot {
    background: var(--status-warning);
}

.status-item.error .status-dot {
    background: var(--status-error);
}

.status-label {
    font-size: var(--text-xs);
}

.status-version {
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    color: var(--text-tertiary);
}

.status-time {
    font-family: var(--font-mono);
    font-size: var(--text-xs);
    color: var(--text-tertiary);
    min-width: 64px;
    text-align: right;
}
</style>
