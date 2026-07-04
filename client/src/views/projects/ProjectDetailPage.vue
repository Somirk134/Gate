<template>
  <div class="project-detail">
    <!-- Header -->
    <div class="pd-header">
      <div class="pd-header-left">
        <button class="btn-icon" @click="$router.push('/projects')">
          <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M10 3L5 8l5 5"/></svg>
        </button>
        <div>
          <h1 class="pd-title">{{ project.name }}</h1>
          <p class="pd-desc">{{ project.description }}</p>
        </div>
      </div>
      <div class="pd-header-right">
        <button class="btn btn-secondary">Start All</button>
        <button class="btn btn-secondary">Stop All</button>
        <button class="btn-icon">
          <svg viewBox="0 0 16 16" fill="currentColor"><circle cx="3" cy="8" r="1.5"/><circle cx="8" cy="8" r="1.5"/><circle cx="13" cy="8" r="1.5"/></svg>
        </button>
      </div>
    </div>

    <!-- Stats Bar -->
    <div class="pd-stats">
      <span class="pd-stat">{{ project.tunnelCount }} Tunnels</span>
      <span class="pd-stat-sep">·</span>
      <span class="pd-stat status-online-text">{{ onlineCount }} Online</span>
      <span class="pd-stat-sep">·</span>
      <span class="pd-stat">2.1 GB Today</span>
    </div>

    <!-- Tunnel Grid + Inspector -->
    <div class="pd-content" :class="{ 'has-inspector': selectedTunnel }">
      <div class="tunnel-grid" :class="{ 'cols-1': !!selectedTunnel }">
        <div
          v-for="tunnel in tunnels"
          :key="tunnel.id"
          class="tunnel-card"
          :class="{ selected: selectedTunnel?.id === tunnel.id, [`status-${tunnel.status}`]: true }"
          @click="selectTunnel(tunnel)"
        >
          <div class="tc-status-bar"></div>
          <div class="tc-body">
            <div class="tc-top">
              <div class="tc-status-badge" :class="tunnel.status">
                <span class="status-dot-sm"></span>
                {{ tunnel.status }}
              </div>
              <button class="tc-more" @click.stop>
                <svg viewBox="0 0 16 16" fill="currentColor"><circle cx="3" cy="8" r="1.3"/><circle cx="8" cy="8" r="1.3"/><circle cx="13" cy="8" r="1.3"/></svg>
              </button>
            </div>
            <h3 class="tc-name">{{ tunnel.name }}</h3>
            <div class="tc-protocol">
              <span class="badge">{{ tunnel.protocol }}</span>
              <span class="tc-address">{{ tunnel.localAddr }} → {{ tunnel.remoteAddr }}</span>
            </div>
            <div class="tc-public">{{ tunnel.publicAddr }}</div>
            <div class="tc-metrics">
              <span class="tc-metric">↓ {{ tunnel.downSpeed }}</span>
              <span class="tc-metric">↑ {{ tunnel.upSpeed }}</span>
              <span class="tc-metric">Con: {{ tunnel.connections }}</span>
            </div>
            <div class="tc-actions">
              <button v-if="tunnel.status === 'online'" class="btn btn-sm btn-danger" @click.stop="tunnel.status = 'offline'">Stop</button>
              <button v-else class="btn btn-sm btn-primary" @click.stop="tunnel.status = 'online'">Start</button>
              <button class="btn btn-sm btn-secondary" @click.stop>Restart</button>
            </div>
          </div>
        </div>
      </div>

      <!-- Inspector -->
      <aside v-if="selectedTunnel" class="inspector-panel">
        <div class="insp-header">
          <h3>Details</h3>
          <button class="btn-icon" @click="selectedTunnel = null">
            <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M4 4l8 8M12 4l-8 8"/></svg>
          </button>
        </div>
        <div class="insp-body">
          <div class="insp-group">
            <span class="insp-label">Status</span>
            <span class="insp-value" :class="selectedTunnel.status">{{ selectedTunnel.status }}</span>
          </div>
          <div class="insp-group">
            <span class="insp-label">Protocol</span>
            <span class="insp-value">{{ selectedTunnel.protocol }}</span>
          </div>
          <div class="insp-group">
            <span class="insp-label">Local</span>
            <span class="insp-value mono">{{ selectedTunnel.localAddr }}</span>
          </div>
          <div class="insp-group">
            <span class="insp-label">Remote</span>
            <span class="insp-value mono">{{ selectedTunnel.remoteAddr }}</span>
          </div>
          <div class="insp-group">
            <span class="insp-label">Public</span>
            <span class="insp-value mono">{{ selectedTunnel.publicAddr }}</span>
          </div>
          <div class="insp-divider"></div>
          <div class="insp-group">
            <span class="insp-label">Bandwidth</span>
            <span class="insp-value">↓ {{ selectedTunnel.downSpeed }} / ↑ {{ selectedTunnel.upSpeed }}</span>
          </div>
          <div class="insp-group">
            <span class="insp-label">Connections</span>
            <span class="insp-value">{{ selectedTunnel.connections }}</span>
          </div>
          <div class="insp-divider"></div>
          <div class="insp-actions">
            <button v-if="selectedTunnel.status === 'online'" class="btn btn-sm btn-danger btn-block" @click="selectedTunnel.status = 'offline'">Stop Tunnel</button>
            <button v-else class="btn btn-sm btn-primary btn-block" @click="selectedTunnel.status = 'online'">Start Tunnel</button>
            <button class="btn btn-sm btn-secondary btn-block">Restart</button>
            <button class="btn btn-sm btn-secondary btn-block">Edit Config</button>
            <button class="btn btn-sm btn-secondary btn-block">View Logs</button>
          </div>
        </div>
      </aside>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRoute } from 'vue-router'

const route = useRoute()
const projectId = computed(() => route.params.projectId as string)

const project = ref({
  id: '1', icon: '📦', name: 'My API Service', description: 'Backend microservices and internal APIs',
  tunnelCount: 5, lastActive: '2 hours ago',
})

const tunnels = ref([
  { id: '1', name: 'api-gateway', protocol: 'TCP', localAddr: 'localhost:3000', remoteAddr: ':8080', publicAddr: 'api.example.com:443', status: 'online', downSpeed: '24 KB/s', upSpeed: '12 KB/s', connections: 8 },
  { id: '2', name: 'database', protocol: 'TCP', localAddr: 'localhost:5432', remoteAddr: ':5432', publicAddr: 'db.example.com:5432', status: 'online', downSpeed: '8 KB/s', upSpeed: '3 KB/s', connections: 2 },
  { id: '3', name: 'web-frontend', protocol: 'HTTP', localAddr: 'localhost:5173', remoteAddr: ':80', publicAddr: 'web.example.com', status: 'offline', downSpeed: '--', upSpeed: '--', connections: 0 },
  { id: '4', name: 'redis-cache', protocol: 'TCP', localAddr: 'localhost:6379', remoteAddr: ':6379', publicAddr: 'cache.example.com:6379', status: 'starting', downSpeed: '--', upSpeed: '--', connections: 0 },
  { id: '5', name: 'admin-panel', protocol: 'HTTP', localAddr: 'localhost:3001', remoteAddr: ':3000', publicAddr: 'admin.example.com', status: 'error', downSpeed: '--', upSpeed: '--', connections: 0 },
])

const selectedTunnel = ref<typeof tunnels.value[0] | null>(null)

const onlineCount = computed(() => tunnels.value.filter(t => t.status === 'online').length)

function selectTunnel(tunnel: typeof tunnels.value[0]) {
  selectedTunnel.value = selectedTunnel.value?.id === tunnel.id ? null : tunnel
}
</script>

<style scoped>
.project-detail { max-width: 1100px; margin: 0 auto; }

/* ── Header ── */
.pd-header { display: flex; align-items: flex-start; justify-content: space-between; margin-bottom: var(--space-3); }
.pd-header-left { display: flex; align-items: flex-start; gap: var(--space-3); }
.pd-title { font-size: var(--text-xl); font-weight: 600; color: var(--text-primary); }
.pd-desc { font-size: var(--text-base); color: var(--text-secondary); margin-top: 2px; }
.pd-header-right { display: flex; gap: var(--space-2); }

/* ── Stats ── */
.pd-stats { display: flex; align-items: center; gap: var(--space-2); margin-bottom: var(--space-5); font-size: var(--text-sm); color: var(--text-muted); padding-bottom: var(--space-3); border-bottom: 1px solid var(--border-subtle); }
.pd-stat-sep { color: var(--border-default); }
.status-online-text { color: var(--status-online); }

/* ── Content ── */
.pd-content { display: flex; gap: var(--space-5); }
.pd-content.has-inspector { }

.tunnel-grid { flex: 1; display: grid; grid-template-columns: repeat(2, 1fr); gap: var(--space-3); align-content: start; }
.tunnel-grid.cols-1 { grid-template-columns: 1fr; }

/* ── Tunnel Card ── */
.tunnel-card { display: flex; background: var(--bg-surface); border: 1px solid var(--border-default); border-radius: var(--radius-lg); overflow: hidden; cursor: pointer; transition: all var(--duration-standard) var(--ease-out); }
.tunnel-card:hover { border-color: var(--border-strong); box-shadow: var(--shadow-xs); transform: translateY(-1px); }
.tunnel-card.selected { border-color: var(--border-accent); box-shadow: 0 0 0 1px var(--border-accent); }
.tc-status-bar { width: 3px; flex-shrink: 0; }
.status-online .tc-status-bar { background: var(--status-online); }
.status-offline .tc-status-bar { background: var(--status-offline); }
.status-starting .tc-status-bar { background: var(--status-starting); }
.status-error .tc-status-bar { background: var(--status-error); }

.tc-body { flex: 1; padding: var(--space-4); min-width: 0; }
.tc-top { display: flex; align-items: center; justify-content: space-between; margin-bottom: var(--space-2); }
.tc-status-badge { display: inline-flex; align-items: center; gap: var(--space-1); padding: 1px var(--space-2); border-radius: var(--radius-sm); font-size: var(--text-xs); font-weight: 500; text-transform: capitalize; }
.tc-status-badge.online { background: var(--status-online-bg); color: var(--status-online); }
.tc-status-badge.offline { background: var(--status-offline-bg); color: var(--status-offline); }
.tc-status-badge.starting { background: var(--status-starting-bg); color: var(--status-starting); }
.tc-status-badge.error { background: var(--status-error-bg); color: var(--status-error); }
.status-dot-sm { width: 5px; height: 5px; border-radius: var(--radius-full); background: currentColor; }

.tc-more { width: 24px; height: 24px; display: flex; align-items: center; justify-content: center; border: none; background: transparent; color: var(--text-muted); border-radius: var(--radius-sm); cursor: pointer; }
.tc-more:hover { background: var(--bg-surface-hover); }
.tc-more svg { width: 12px; height: 12px; }

.tc-name { font-size: var(--text-md); font-weight: 600; color: var(--text-primary); margin-bottom: var(--space-1); }
.tc-protocol { display: flex; align-items: center; gap: var(--space-2); margin-bottom: var(--space-1); }
.tc-address { font-size: var(--text-sm); color: var(--text-secondary); font-family: var(--font-mono); }
.tc-public { font-size: var(--text-xs); color: var(--text-muted); font-family: var(--font-mono); margin-bottom: var(--space-2); }
.tc-metrics { display: flex; gap: var(--space-3); margin-bottom: var(--space-3); font-size: var(--text-xs); color: var(--text-muted); font-family: var(--font-mono); }
.tc-actions { display: flex; gap: var(--space-2); }

/* ── Inspector ── */
.inspector-panel { width: 300px; flex-shrink: 0; background: var(--bg-surface); border: 1px solid var(--border-default); border-radius: var(--radius-lg); align-self: start; position: sticky; top: 0; }
.insp-header { display: flex; align-items: center; justify-content: space-between; padding: var(--space-3) var(--space-4); border-bottom: 1px solid var(--border-subtle); }
.insp-header h3 { font-size: var(--text-sm); font-weight: 500; color: var(--text-secondary); }
.insp-body { padding: var(--space-4); }
.insp-group { display: flex; justify-content: space-between; align-items: center; margin-bottom: var(--space-2); }
.insp-label { font-size: var(--text-sm); color: var(--text-muted); }
.insp-value { font-size: var(--text-sm); color: var(--text-primary); font-weight: 500; }
.insp-value.mono { font-family: var(--font-mono); font-weight: 400; }
.insp-value.online { color: var(--status-online); }
.insp-value.error { color: var(--status-error); }
.insp-value.offline { color: var(--status-offline); }
.insp-divider { height: 1px; background: var(--border-subtle); margin: var(--space-3) 0; }
.insp-actions { display: flex; flex-direction: column; gap: var(--space-2); }

/* ── Badge ── */
.badge { display: inline-block; padding: 1px 6px; background: var(--accent-primary-muted); color: var(--accent-primary); border-radius: var(--radius-sm); font-size: var(--text-xs); font-weight: 500; }

/* ── Buttons ── */
.btn { display: inline-flex; align-items: center; justify-content: center; gap: var(--space-2); height: 32px; padding: 0 var(--space-3); border: 1px solid var(--border-default); background: transparent; color: var(--text-primary); font-family: var(--font-ui); font-size: var(--text-base); font-weight: 500; border-radius: var(--radius-md); cursor: pointer; transition: all var(--duration-micro) var(--ease-out); white-space: nowrap; }
.btn:hover { background: var(--bg-surface-hover); border-color: var(--border-strong); }
.btn:active { transform: scale(0.97); }
.btn-primary { background: var(--accent-primary); border-color: var(--accent-primary); color: white; }
.btn-primary:hover { background: var(--accent-primary-hover); }
.btn-secondary { background: transparent; border-color: var(--border-default); color: var(--text-secondary); }
.btn-danger { color: var(--status-error); border-color: rgba(239,68,68,0.3); background: rgba(239,68,68,0.1); }
.btn-danger:hover { background: rgba(239,68,68,0.2); }
.btn-sm { height: 28px; padding: 0 var(--space-2); font-size: var(--text-sm); }
.btn-block { width: 100%; }
.btn-icon { width: 32px; height: 32px; display: flex; align-items: center; justify-content: center; border: none; background: transparent; color: var(--text-muted); border-radius: var(--radius-md); cursor: pointer; }
.btn-icon:hover { background: var(--bg-surface-hover); color: var(--text-primary); }
.btn-icon svg { width: 16px; height: 16px; }
</style>
