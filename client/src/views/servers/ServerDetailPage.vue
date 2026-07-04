<template>
  <div class="server-detail">
    <div class="sd-header">
      <button class="btn-icon" @click="$router.push('/servers')">
        <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M10 3L5 8l5 5"/></svg>
      </button>
      <div>
        <h1 class="sd-title">{{ server.name }} <span class="status-dot" :class="server.status"></span></h1>
        <p class="sd-subtitle">{{ server.ip }} · {{ server.region }} · {{ server.version }}</p>
      </div>
    </div>

    <!-- Metrics -->
    <div class="sd-metrics">
      <div class="metric-card">
        <span class="metric-label">CPU</span>
        <div class="metric-bar"><div class="metric-fill" style="width:12%"></div></div>
        <span class="metric-value">12%</span>
      </div>
      <div class="metric-card">
        <span class="metric-label">RAM</span>
        <div class="metric-bar"><div class="metric-fill ram" style="width:34%"></div></div>
        <span class="metric-value">34%</span>
      </div>
      <div class="metric-card">
        <span class="metric-label">Disk</span>
        <div class="metric-bar"><div class="metric-fill disk" style="width:45%"></div></div>
        <span class="metric-value">45%</span>
      </div>
      <div class="metric-card">
        <span class="metric-label">Network</span>
        <span class="metric-value">↓24 ↑12 KB/s</span>
      </div>
    </div>

    <!-- Actions -->
    <div class="sd-actions">
      <button class="btn btn-secondary">Reconnect</button>
      <button class="btn btn-danger">Remove Server</button>
    </div>

    <!-- Server Logs -->
    <div class="sd-logs">
      <div class="section-header">
        <h2>Server Logs</h2>
      </div>
      <div class="log-console">
        <div v-for="log in logs" :key="log.id" class="log-entry" :class="`log-${log.level}`">
          <span class="log-time">{{ log.time }}</span>
          <span class="log-level">{{ log.level.toUpperCase() }}</span>
          <span class="log-msg">{{ log.message }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

const server = ref({ name: 'us-west-1', ip: '54.183.45.12', region: 'San Francisco, US', ping: 24, version: 'v0.1.0', status: 'online', cpu: 12, ram: 34 })

const logs = ref([
  { id: '1', time: '12:34:56', level: 'info', message: 'Server started on 0.0.0.0:8080' },
  { id: '2', time: '12:35:00', level: 'info', message: 'Client connected from 192.168.1.100' },
  { id: '3', time: '12:35:01', level: 'info', message: 'Tunnel "api-gateway" registered' },
  { id: '4', time: '12:36:00', level: 'warn', message: 'High latency detected on connection #42' },
])
</script>

<style scoped>
.server-detail { max-width: 960px; margin: 0 auto; }

.sd-header { display: flex; align-items: flex-start; gap: var(--space-3); margin-bottom: var(--space-5); }
.sd-title { font-size: var(--text-xl); font-weight: 600; color: var(--text-primary); display: flex; align-items: center; gap: var(--space-2); }
.sd-subtitle { font-size: var(--text-base); color: var(--text-secondary); margin-top: 2px; }
.status-dot { width: 10px; height: 10px; border-radius: var(--radius-full); display: inline-block; }
.status-dot.online { background: var(--status-online); }
.status-dot.offline { background: var(--status-offline); }

.sd-metrics { display: grid; grid-template-columns: repeat(4, 1fr); gap: var(--space-3); margin-bottom: var(--space-5); }
.metric-card { padding: var(--space-4); background: var(--bg-surface); border: 1px solid var(--border-default); border-radius: var(--radius-lg); }
.metric-label { display: block; font-size: var(--text-xs); color: var(--text-muted); text-transform: uppercase; letter-spacing: var(--tracking-wide); margin-bottom: var(--space-2); }
.metric-bar { height: 4px; background: var(--bg-input); border-radius: var(--radius-full); margin-bottom: var(--space-1); overflow: hidden; }
.metric-fill { height: 100%; background: var(--accent-primary); border-radius: var(--radius-full); }
.metric-fill.ram { background: var(--accent-secondary); }
.metric-fill.disk { background: var(--status-warning); }
.metric-value { font-size: var(--text-base); font-weight: 500; color: var(--text-primary); font-family: var(--font-mono); }

.sd-actions { display: flex; gap: var(--space-2); margin-bottom: var(--space-6); }

.section-header { margin-bottom: var(--space-3); }
.section-header h2 { font-size: var(--text-lg); font-weight: 600; }

.log-console { background: var(--bg-app); border: 1px solid var(--border-default); border-radius: var(--radius-lg); padding: var(--space-3); font-family: var(--font-mono); font-size: var(--text-xs); line-height: var(--leading-relaxed); max-height: 300px; overflow-y: auto; }
.log-entry { display: flex; gap: var(--space-2); padding: 2px 4px; border-radius: 2px; }
.log-entry.log-warn { background: var(--status-warning-bg); }
.log-time { color: var(--text-muted); flex-shrink: 0; }
.log-level { font-weight: 600; flex-shrink: 0; width: 44px; }
.log-entry.log-info .log-level { color: var(--status-info); }
.log-entry.log-warn .log-level { color: var(--status-warning); }
.log-entry.log-error .log-level { color: var(--status-error); }
.log-msg { color: var(--text-secondary); }

.btn { display: inline-flex; align-items: center; gap: var(--space-2); height: 32px; padding: 0 var(--space-4); border: 1px solid var(--border-default); background: transparent; color: var(--text-primary); font-family: var(--font-ui); font-size: var(--text-base); font-weight: 500; border-radius: var(--radius-md); cursor: pointer; transition: all var(--duration-micro) var(--ease-out); }
.btn:hover { background: var(--bg-surface-hover); border-color: var(--border-strong); }
.btn-secondary { background: transparent; border-color: var(--border-default); color: var(--text-secondary); }
.btn-danger { color: var(--status-error); border-color: rgba(239,68,68,0.3); background: rgba(239,68,68,0.1); }
.btn-danger:hover { background: rgba(239,68,68,0.2); }
.btn-icon { width: 32px; height: 32px; display: flex; align-items: center; justify-content: center; border: none; background: transparent; color: var(--text-muted); border-radius: var(--radius-md); cursor: pointer; }
.btn-icon:hover { background: var(--bg-surface-hover); color: var(--text-primary); }
.btn-icon svg { width: 16px; height: 16px; }
</style>
