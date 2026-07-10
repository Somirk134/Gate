<template>
  <footer class="app-statusbar">
    <div class="statusbar-left">
      <div class="status-item" :class="statusClass">
        <span class="status-dot" />
        <span class="status-label">{{ t(statusLabelKey) }}</span>
      </div>
      <div v-if="connectionStatus" class="status-item">
        <GIcon name="wifi" :size="12" />
        <span class="status-label">{{ t('common.connected') }}</span>
      </div>
    </div>

    <div class="statusbar-center">
      <span class="status-version">Gate v{{ version }}</span>
    </div>

    <div class="statusbar-right">
      <span class="status-item">{{ t('common.cpu') }}: --</span>
      <span class="status-item">{{ t('common.memory') }}: --</span>
      <span class="status-item">{{ t('common.traffic') }}: --</span>
      <span class="status-item status-time">{{ currentTime }}</span>
    </div>
  </footer>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import GIcon from '@components/icons/GIcon.vue'
import { APP_VERSION } from '@/constants'

const version = APP_VERSION
const { t, locale } = useI18n()
const statusLabelKey = ref('common.ready')
const statusClass = ref('online')
const connectionStatus = ref(true)
const currentTime = ref('')

let timer: number

function updateTime() {
  const now = new Date()
  currentTime.value = now.toLocaleTimeString(locale.value === 'en-US' ? 'en-US' : 'zh-CN', {
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
