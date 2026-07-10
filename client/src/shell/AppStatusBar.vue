<template>
  <footer class="app-statusbar">
    <div class="statusbar-left">
      <div class="status-item" :class="statusClass">
        <span class="status-dot" />
        <span class="status-label">{{ statusLabel }}</span>
      </div>
      <div class="status-item">
        <GIcon name="wifi" :size="12" />
        <span class="status-label">{{ formatLatency(dashboard.statistics.connection.averageRttMs) }}</span>
      </div>
    </div>

    <div class="statusbar-center">
      <span class="status-version">Gate v{{ version }}</span>
    </div>

    <div class="statusbar-right">
      <span class="status-item">{{ t('common.cpu') }}: {{ dashboard.statistics.system.cpuUsage.toFixed(0) }}%</span>
      <span class="status-item">{{ t('common.memory') }}: {{ dashboard.statistics.system.memoryUsage.toFixed(0) }}%</span>
      <span class="status-item">Up: {{ formatSpeed(dashboard.statistics.traffic.uploadSpeedBps) }}</span>
      <span class="status-item">Down: {{ formatSpeed(dashboard.statistics.traffic.downloadSpeedBps) }}</span>
      <span class="status-item">Log: {{ dashboard.recentActivity.length }}</span>
      <span class="status-item status-time">{{ currentTime }}</span>
    </div>
  </footer>
</template>

<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from 'vue'
import { useI18n } from 'vue-i18n'
import GIcon from '@components/icons/GIcon.vue'
import { APP_VERSION } from '@/constants'
import { useMonitoringDashboard } from '@/monitoring/composables/useMonitoringDashboard'

const version = APP_VERSION
const { t, locale } = useI18n()
const { dashboard, healthStatus } = useMonitoringDashboard()
const currentTime = ref('')
const statusLabel = computed(() => t(`dashboard.healthStatus.${healthStatus.value}`))
const statusClass = computed(() => {
  if (healthStatus.value === 'healthy') return 'online'
  if (healthStatus.value === 'warning') return 'warning'
  if (healthStatus.value === 'critical') return 'error'
  return 'offline'
})

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

function formatLatency(milliseconds: number): string {
  if (!Number.isFinite(milliseconds) || milliseconds <= 0) return '0 ms'
  return `${Math.round(milliseconds)} ms`
}

function formatSpeed(bytesPerSecond: number): string {
  if (!Number.isFinite(bytesPerSecond) || bytesPerSecond <= 0) return '0 B/s'
  const units = ['B/s', 'KB/s', 'MB/s', 'GB/s']
  const index = Math.min(units.length - 1, Math.floor(Math.log(bytesPerSecond) / Math.log(1024)))
  const value = bytesPerSecond / 1024 ** index
  return `${value.toFixed(value >= 10 || index === 0 ? 0 : 1)} ${units[index]}`
}
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
  min-width: 0;
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
  min-width: 0;
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

@media (max-width: 980px) {
  .statusbar-center,
  .statusbar-right .status-item:nth-child(n + 4):not(.status-time) {
    display: none;
  }
}

@media (max-width: 720px) {
  .statusbar-right .status-item:not(.status-time) {
    display: none;
  }
}
</style>
