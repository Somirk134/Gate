<template>
  <div class="logs-page">
    <div class="page-header">
      <h1 class="page-title">{{ t('logs.title') }}</h1>
      <div class="header-actions">
        <button class="btn btn-secondary btn-sm" @click="paused = !paused">
          {{ paused ? t('logs.resume') : t('logs.pause') }}
        </button>
        <button class="btn btn-secondary btn-sm">⬇ {{ t('logs.export') }}</button>
      </div>
    </div>

    <div class="log-toolbar">
      <div class="search-box">
        <svg viewBox="0 0 18 18" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="8" cy="8" r="5.5"/><line x1="12.5" y1="12.5" x2="16" y2="16"/></svg>
        <input v-model="searchQuery" type="text" :placeholder="t('logs.search')" class="search-input" />
      </div>
      <select v-model="levelFilter" class="select-input">
        <option value="all">{{ t('logs.allLevels') }}</option>
        <option value="info">{{ t('logs.level.info') }}</option>
        <option value="warn">{{ t('logs.level.warn') }}</option>
        <option value="error">{{ t('logs.level.error') }}</option>
        <option value="debug">{{ t('logs.level.debug') }}</option>
      </select>
      <select v-model="sourceFilter" class="select-input">
        <option value="all">{{ t('logs.allSources') }}</option>
        <option value="server">gate::server</option>
        <option value="tunnel">gate::tunnel</option>
        <option value="connection">gate::connection</option>
        <option value="auth">gate::auth</option>
      </select>
    </div>

    <div class="log-console" ref="consoleRef">
      <div v-for="log in filteredLogs" :key="log.id" class="log-entry" :class="`log-${log.level}`">
        <span class="log-time">{{ log.time }}</span>
        <span class="log-level">{{ log.level.toUpperCase() }}</span>
        <span class="log-source">{{ log.source }}</span>
        <span class="log-msg" v-html="highlightMatch(log.message)"></span>
      </div>
      <div v-if="filteredLogs.length === 0" class="log-empty">{{ t('logs.empty') }}</div>
    </div>

    <div class="log-footer">
      <span>{{ filteredLogs.length }} {{ t('logs.entries') }}</span>
      <span>{{ t('logs.autoScroll') }}: {{ paused ? t('logs.autoScrollOff') : t('logs.autoScrollOn') }}</span>
      <span v-if="paused && newLogCount" class="new-logs-badge">{{ t('logs.newCount', { count: newLogCount }) }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const searchQuery = ref('')
const levelFilter = ref('all')
const sourceFilter = ref('all')
const paused = ref(false)
const newLogCount = ref(0)
const consoleRef = ref<HTMLElement | null>(null)

const logs = ref<Array<{ id: string; time: string; level: string; source: string; message: string }>>([
  { id: '1', time: '12:34:56.789', level: 'info', source: 'gate::server', message: 'Server started on 0.0.0.0:8080' },
  { id: '2', time: '12:34:57.012', level: 'debug', source: 'gate::tunnel', message: 'Loading tunnel configurations...' },
  { id: '3', time: '12:34:58.345', level: 'info', source: 'gate::tunnel', message: 'Tunnel "api-gateway" is now online' },
  { id: '4', time: '12:35:01.123', level: 'warn', source: 'gate::connection', message: 'High latency detected: 245ms on connection #42' },
  { id: '5', time: '12:35:05.678', level: 'error', source: 'gate::auth', message: 'Authentication token expired for client 192.168.1.100' },
  { id: '6', time: '12:35:10.001', level: 'info', source: 'gate::server', message: 'Health check passed: all services running' },
  { id: '7', time: '12:35:15.234', level: 'debug', source: 'gate::tunnel', message: 'Tunnel "web-frontend" configuration validated' },
  { id: '8', time: '12:35:20.456', level: 'info', source: 'gate::connection', message: 'New connection accepted from 10.0.0.23:54321' },
])

const filteredLogs = computed(() => {
  let result = logs.value
  if (levelFilter.value !== 'all') result = result.filter(l => l.level === levelFilter.value)
  if (sourceFilter.value !== 'all') result = result.filter(l => l.source.includes(sourceFilter.value))
  if (searchQuery.value) {
    const q = searchQuery.value.toLowerCase()
    result = result.filter(l => l.message.toLowerCase().includes(q) || l.source.toLowerCase().includes(q))
  }
  return result
})

function highlightMatch(text: string): string {
  if (!searchQuery.value) return escapeHtml(text)
  const q = escapeRegex(searchQuery.value)
  return escapeHtml(text).replace(new RegExp(`(${q})`, 'gi'), '<mark>$1</mark>')
}

function escapeHtml(s: string): string {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;')
}

function escapeRegex(s: string): string {
  return s.replace(/[.*+?^${}()|[\]\\]/g, '\\$&')
}

// Simulate new logs arriving
setInterval(() => {
  if (Math.random() > 0.7) {
    const now = new Date()
    const time = now.toTimeString().slice(0,12)
    const levels = ['info', 'info', 'info', 'debug', 'warn']
    const level = levels[Math.floor(Math.random() * levels.length)]
    logs.value.push({
      id: String(Date.now()),
      time,
      level,
      source: ['gate::server', 'gate::tunnel', 'gate::connection'][Math.floor(Math.random() * 3)],
      message: `Simulated log entry at ${time}`,
    })
    if (paused.value) newLogCount.value++
  }
}, 3000)

watch(filteredLogs, async () => {
  if (!paused.value) {
    await nextTick()
    if (consoleRef.value) consoleRef.value.scrollTop = consoleRef.value.scrollHeight
  }
})
</script>

<style scoped>
.logs-page { max-width: 1100px; margin: 0 auto; height: 100%; display: flex; flex-direction: column; }

.page-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: var(--space-4); }
.page-title { font-size: var(--text-xl); font-weight: 600; }
.header-actions { display: flex; gap: var(--space-2); }

.log-toolbar { display: flex; align-items: center; gap: var(--space-3); margin-bottom: var(--space-3); }

.search-box { display: flex; align-items: center; gap: var(--space-2); flex: 1; max-width: 320px; padding: 0 var(--space-3); height: 32px; background: var(--bg-input); border: 1px solid var(--border-default); border-radius: var(--radius-md); }
.search-box svg { width: 14px; height: 14px; color: var(--text-muted); flex-shrink: 0; }
.search-input { flex: 1; border: none; background: transparent; color: var(--text-primary); font-family: var(--font-ui); font-size: var(--text-base); outline: none; }
.search-input::placeholder { color: var(--text-muted); }

.select-input { height: 32px; padding: 0 var(--space-3); background: var(--bg-input); border: 1px solid var(--border-default); border-radius: var(--radius-md); color: var(--text-secondary); font-family: var(--font-ui); font-size: var(--text-sm); cursor: pointer; }

.log-console { flex: 1; background: var(--bg-app); border: 1px solid var(--border-default); border-radius: var(--radius-lg); padding: var(--space-3); font-family: var(--font-mono); font-size: var(--text-xs); line-height: var(--leading-relaxed); overflow-y: auto; }
.log-entry { display: flex; gap: var(--space-2); padding: 2px 4px; border-radius: 2px; }
.log-entry.log-error { background: var(--status-error-bg); }
.log-entry.log-warn { background: var(--status-warning-bg); }
.log-time { color: var(--text-muted); flex-shrink: 0; }
.log-level { font-weight: 600; flex-shrink: 0; width: 44px; }
.log-entry.log-info .log-level { color: var(--status-info); }
.log-entry.log-warn .log-level { color: var(--status-warning); }
.log-entry.log-error .log-level { color: var(--status-error); }
.log-entry.log-debug .log-level { color: var(--text-muted); }
.log-source { color: var(--text-muted); flex-shrink: 0; }
.log-msg { color: var(--text-secondary); flex: 1; word-break: break-all; }
.log-msg :deep(mark) { background: rgba(245,158,11,0.3); color: var(--text-primary); border-radius: 2px; padding: 0 1px; }
.log-empty { padding: var(--space-6); text-align: center; color: var(--text-muted); }

.log-footer { display: flex; align-items: center; justify-content: space-between; padding: var(--space-2) var(--space-3); font-size: var(--text-xs); color: var(--text-muted); border-top: 1px solid var(--border-subtle); margin-top: var(--space-2); }
.new-logs-badge { background: var(--accent-primary); color: white; padding: 1px 8px; border-radius: var(--radius-full); font-size: 11px; font-weight: 600; }

.btn { display: inline-flex; align-items: center; gap: var(--space-2); height: 32px; padding: 0 var(--space-3); border: 1px solid var(--border-default); background: transparent; color: var(--text-primary); font-family: var(--font-ui); font-size: var(--text-base); font-weight: 500; border-radius: var(--radius-md); cursor: pointer; transition: all var(--duration-micro) var(--ease-out); }
.btn:hover { background: var(--bg-surface-hover); border-color: var(--border-strong); }
.btn-secondary { background: transparent; border-color: var(--border-default); color: var(--text-secondary); }
.btn-sm { height: 28px; padding: 0 var(--space-2); font-size: var(--text-sm); }
</style>
