<template>
  <div class="servers-page">
    <div class="page-header">
      <h1 class="page-title">Servers</h1>
      <button class="btn btn-primary">+ Add Server</button>
    </div>

    <div class="page-toolbar">
      <div class="search-box">
        <svg viewBox="0 0 18 18" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="8" cy="8" r="5.5"/><line x1="12.5" y1="12.5" x2="16" y2="16"/></svg>
        <input v-model="searchQuery" type="text" placeholder="Search servers..." class="search-input" />
      </div>
    </div>

    <div v-if="filteredServers.length" class="server-grid">
      <div v-for="server in filteredServers" :key="server.id" class="server-card" @click="$router.push(`/servers/${server.id}`)">
        <div class="sc-top">
          <div class="sc-icon">🖥</div>
          <div class="sc-status-dot" :class="server.status"></div>
        </div>
        <h3 class="sc-name">{{ server.name }}</h3>
        <div class="sc-ip mono">{{ server.ip }}</div>
        <div class="sc-region">{{ server.region }}</div>
        <div class="sc-meta">
          <span>Ping: {{ server.ping }}ms</span>
          <span class="badge-sm">{{ server.version }}</span>
        </div>
        <div class="sc-resources">
          <div class="sc-resource">
            <span class="sc-res-label">CPU</span>
            <div class="sc-res-bar">
              <div class="sc-res-fill" :style="{ width: server.cpu + '%' }"></div>
            </div>
            <span class="sc-res-val">{{ server.cpu }}%</span>
          </div>
          <div class="sc-resource">
            <span class="sc-res-label">RAM</span>
            <div class="sc-res-bar">
              <div class="sc-res-fill ram" :style="{ width: server.ram + '%' }"></div>
            </div>
            <span class="sc-res-val">{{ server.ram }}%</span>
          </div>
        </div>
      </div>
    </div>

    <div v-else class="empty-state">
      <div class="empty-icon">
        <svg viewBox="0 0 48 48" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="6" y="6" width="36" height="13" rx="2"/><rect x="6" y="23" width="36" height="13" rx="2"/><circle cx="13" cy="12.5" r="2" fill="currentColor"/><circle cx="13" cy="29.5" r="2" fill="currentColor"/></svg>
      </div>
      <h2 class="empty-title">No servers connected</h2>
      <p class="empty-desc">Add a Gate server to start creating tunnels</p>
      <button class="btn btn-primary">Add a server</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

const searchQuery = ref('')

const servers = ref([
  { id: '1', name: 'us-west-1', ip: '54.183.45.12', region: 'San Francisco, US', ping: 24, version: 'v0.1.0', status: 'online', cpu: 12, ram: 34 },
  { id: '2', name: 'eu-central', ip: '18.197.32.8', region: 'Frankfurt, DE', ping: 156, version: 'v0.1.0', status: 'online', cpu: 8, ram: 22 },
  { id: '3', name: 'ap-southeast', ip: '13.229.45.10', region: 'Singapore', ping: 89, version: 'v0.1.0', status: 'offline', cpu: 0, ram: 0 },
])

const filteredServers = computed(() => {
  const q = searchQuery.value.toLowerCase()
  return servers.value.filter(s => s.name.toLowerCase().includes(q) || s.ip.includes(q) || s.region.toLowerCase().includes(q))
})
</script>

<style scoped>
.servers-page { max-width: 1024px; margin: 0 auto; }

.page-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: var(--space-5); }
.page-title { font-size: var(--text-xl); font-weight: 600; color: var(--text-primary); }

.page-toolbar { margin-bottom: var(--space-5); }

.search-box { display: flex; align-items: center; gap: var(--space-2); max-width: 320px; padding: 0 var(--space-3); height: 32px; background: var(--bg-input); border: 1px solid var(--border-default); border-radius: var(--radius-md); }
.search-box svg { width: 14px; height: 14px; color: var(--text-muted); flex-shrink: 0; }
.search-input { flex: 1; border: none; background: transparent; color: var(--text-primary); font-family: var(--font-ui); font-size: var(--text-base); outline: none; }
.search-input::placeholder { color: var(--text-muted); }

.server-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: var(--space-4); }

.server-card { padding: var(--space-5); background: var(--bg-surface); border: 1px solid var(--border-default); border-radius: var(--radius-xl); cursor: pointer; transition: all var(--duration-standard) var(--ease-out); }
.server-card:hover { border-color: var(--border-strong); background: var(--bg-surface-hover); transform: translateY(-2px); box-shadow: var(--shadow-sm); }

.sc-top { display: flex; align-items: center; justify-content: space-between; margin-bottom: var(--space-3); }
.sc-icon { font-size: 28px; }
.sc-status-dot { width: 10px; height: 10px; border-radius: var(--radius-full); }
.sc-status-dot.online { background: var(--status-online); }
.sc-status-dot.offline { background: var(--status-offline); }

.sc-name { font-size: var(--text-lg); font-weight: 600; color: var(--text-primary); margin-bottom: var(--space-1); }
.sc-ip { font-size: var(--text-sm); color: var(--text-secondary); margin-bottom: 2px; }
.sc-region { font-size: var(--text-sm); color: var(--text-muted); margin-bottom: var(--space-3); }

.sc-meta { display: flex; align-items: center; gap: var(--space-3); margin-bottom: var(--space-3); font-size: var(--text-xs); color: var(--text-muted); }

.sc-resources { display: flex; flex-direction: column; gap: var(--space-2); }
.sc-resource { display: flex; align-items: center; gap: var(--space-2); }
.sc-res-label { font-size: var(--text-xs); color: var(--text-muted); width: 28px; }
.sc-res-bar { flex: 1; height: 4px; background: var(--bg-input); border-radius: var(--radius-full); overflow: hidden; }
.sc-res-fill { height: 100%; background: var(--accent-primary); border-radius: var(--radius-full); transition: width 2s var(--ease-out); }
.sc-res-fill.ram { background: var(--accent-secondary); }
.sc-res-val { font-size: var(--text-xs); color: var(--text-secondary); width: 32px; text-align: right; font-family: var(--font-mono); }

/* ── Badge ── */
.badge-sm { display: inline-block; padding: 1px 6px; background: var(--accent-primary-muted); color: var(--accent-primary); border-radius: var(--radius-sm); font-size: 11px; font-weight: 500; font-family: var(--font-mono); }

/* ── Buttons ── */
.btn { display: inline-flex; align-items: center; gap: var(--space-2); height: 32px; padding: 0 var(--space-4); border: 1px solid var(--border-default); background: transparent; color: var(--text-primary); font-family: var(--font-ui); font-size: var(--text-base); font-weight: 500; border-radius: var(--radius-md); cursor: pointer; transition: all var(--duration-micro) var(--ease-out); }
.btn:hover { background: var(--bg-surface-hover); border-color: var(--border-strong); }
.btn:active { transform: scale(0.97); }
.btn-primary { background: var(--accent-primary); border-color: var(--accent-primary); color: white; }
.btn-primary:hover { background: var(--accent-primary-hover); }

.mono { font-family: var(--font-mono); }

.empty-state { display: flex; flex-direction: column; align-items: center; justify-content: center; padding: var(--space-12) var(--space-6); text-align: center; }
.empty-icon { width: 64px; height: 64px; color: var(--text-muted); margin-bottom: var(--space-4); }
.empty-icon svg { width: 100%; height: 100%; }
.empty-title { font-size: var(--text-lg); font-weight: 600; color: var(--text-primary); margin-bottom: var(--space-2); }
.empty-desc { font-size: var(--text-base); color: var(--text-muted); margin-bottom: var(--space-5); }
</style>
