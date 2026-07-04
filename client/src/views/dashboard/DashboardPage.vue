<template>
  <div class="dashboard">
    <!-- Welcome Banner -->
    <div class="welcome-banner">
      <div class="welcome-content">
        <h1 class="welcome-greeting">{{ greeting }}</h1>
        <p class="welcome-subtitle">Your server is healthy · <span class="text-accent">{{ onlineTunnels }}</span> tunnels online</p>
      </div>
      <div class="welcome-actions">
        <button class="btn btn-primary" @click="$router.push('/projects')">View Projects</button>
      </div>
    </div>

    <!-- Quick Actions -->
    <div class="quick-actions">
      <button class="action-card" @click="$router.push('/projects?new=1')">
        <span class="action-icon">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="3" y="3" width="18" height="18" rx="2"/><line x1="12" y1="8" x2="12" y2="16"/><line x1="8" y1="12" x2="16" y2="12"/></svg>
        </span>
        <span class="action-label">New Project</span>
        <span class="action-shortcut">⌘N</span>
      </button>
      <button class="action-card">
        <span class="action-icon">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/></svg>
        </span>
        <span class="action-label">Quick Tunnel</span>
        <span class="action-shortcut">⌘T</span>
      </button>
      <button class="action-card" @click="$router.push('/servers')">
        <span class="action-icon">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="2" y="2" width="20" height="8" rx="2"/><rect x="2" y="14" width="20" height="8" rx="2"/><circle cx="6" cy="6" r="1" fill="currentColor"/><circle cx="6" cy="18" r="1" fill="currentColor"/></svg>
        </span>
        <span class="action-label">Server Status</span>
      </button>
      <button class="action-card">
        <span class="action-icon">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="M9.09 9a3 3 0 0 1 5.83 1c0 2-3 3-3 3"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg>
        </span>
        <span class="action-label">Quick Help</span>
      </button>
    </div>

    <!-- Stats Row -->
    <div class="stats-row">
      <div class="stat-card">
        <span class="stat-value">{{ totalTunnels }}</span>
        <span class="stat-label">Total Tunnels</span>
      </div>
      <div class="stat-card">
        <span class="stat-value status-online">{{ onlineTunnels }}</span>
        <span class="stat-label">Online</span>
      </div>
      <div class="stat-card">
        <span class="stat-value">{{ totalServers }}</span>
        <span class="stat-label">Servers</span>
      </div>
      <div class="stat-card">
        <span class="stat-value">2.1 GB</span>
        <span class="stat-label">Today Traffic</span>
      </div>
    </div>

    <!-- Recent Projects & Logs -->
    <div class="bottom-row">
      <section class="section-panel">
        <div class="section-header">
          <h2 class="section-title">Recent Projects</h2>
          <button class="btn btn-ghost btn-sm" @click="$router.push('/projects')">View All →</button>
        </div>
        <div class="recent-projects" v-if="recentProjects.length">
          <div
            v-for="project in recentProjects"
            :key="project.id"
            class="recent-project-item"
            @click="$router.push(`/projects/${project.id}`)"
          >
            <span class="rp-icon">{{ project.icon }}</span>
            <div class="rp-info">
              <span class="rp-name">{{ project.name }}</span>
              <span class="rp-meta">{{ project.tunnelCount }} tunnels · {{ project.onlineCount }} online</span>
            </div>
            <span class="rp-time">{{ project.lastActive }}</span>
          </div>
        </div>
        <div v-else class="empty-placeholder">
          <p>No projects yet</p>
          <button class="btn btn-primary btn-sm">Create your first project</button>
        </div>
      </section>

      <section class="section-panel">
        <div class="section-header">
          <h2 class="section-title">Recent Logs</h2>
          <button class="btn btn-ghost btn-sm" @click="$router.push('/logs')">View All →</button>
        </div>
        <div class="recent-logs" v-if="recentLogs.length">
          <div v-for="log in recentLogs" :key="log.id" class="log-line" :class="`log-${log.level}`">
            <span class="log-time">{{ log.time }}</span>
            <span class="log-level">{{ log.level.toUpperCase() }}</span>
            <span class="log-source">{{ log.source }}</span>
            <span class="log-msg">{{ log.message }}</span>
          </div>
        </div>
        <div v-else class="empty-placeholder">
          <p>No recent logs</p>
        </div>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

const onlineTunnels = ref(3)
const totalTunnels = ref(8)
const totalServers = ref(2)

const greeting = computed(() => {
  const hour = new Date().getHours()
  if (hour < 12) return '👋 Good morning'
  if (hour < 18) return '☀️ Good afternoon'
  return '🌙 Good evening'
})

const recentProjects = ref([
  { id: '1', icon: '📦', name: 'My API Service', tunnelCount: 8, onlineCount: 3, lastActive: '2 hours ago' },
  { id: '2', icon: '🌐', name: 'Web App Frontend', tunnelCount: 3, onlineCount: 1, lastActive: '1 day ago' },
  { id: '3', icon: '🛠', name: 'Dev Environment', tunnelCount: 5, onlineCount: 5, lastActive: 'Just now' },
  { id: '4', icon: '📊', name: 'Monitoring Stack', tunnelCount: 4, onlineCount: 2, lastActive: '3 hours ago' },
])

const recentLogs = ref([
  { id: '1', time: '12:34:56', level: 'info', source: 'gate::server', message: 'Server started on 0.0.0.0:8080' },
  { id: '2', time: '12:34:57', level: 'debug', source: 'gate::tunnel', message: 'Loading tunnel configurations...' },
  { id: '3', time: '12:34:58', level: 'info', source: 'gate::tunnel', message: 'Tunnel "api-gateway" is now online' },
  { id: '4', time: '12:35:01', level: 'warn', source: 'gate::connection', message: 'High latency detected: 245ms' },
  { id: '5', time: '12:35:05', level: 'error', source: 'gate::auth', message: 'Authentication token expired' },
])
</script>

<style scoped>
.dashboard {
  max-width: 960px;
  margin: 0 auto;
}

/* ── Welcome ── */
.welcome-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-6);
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-xl);
  margin-bottom: var(--space-6);
}

.welcome-greeting {
  font-size: var(--text-xl);
  font-weight: 600;
  color: var(--text-primary);
  letter-spacing: var(--tracking-tight);
  margin-bottom: var(--space-1);
}

.welcome-subtitle {
  font-size: var(--text-base);
  color: var(--text-secondary);
}

.text-accent {
  color: var(--status-online);
  font-weight: 500;
}

/* ── Quick Actions ── */
.quick-actions {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: var(--space-3);
  margin-bottom: var(--space-6);
}

.action-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-5);
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-lg);
  cursor: pointer;
  transition: all var(--duration-standard) var(--ease-out);
  color: var(--text-secondary);
  font-family: var(--font-ui);
}

.action-card:hover {
  border-color: var(--border-strong);
  background: var(--bg-surface-hover);
  transform: scale(1.02);
  color: var(--text-primary);
}

.action-icon {
  width: 28px;
  height: 28px;
  color: var(--accent-primary);
}

.action-icon svg {
  width: 100%;
  height: 100%;
}

.action-label {
  font-size: var(--text-sm);
  font-weight: 500;
}

.action-shortcut {
  font-size: var(--text-xs);
  color: var(--text-muted);
  font-family: var(--font-mono);
  padding: 1px 6px;
  background: var(--bg-input);
  border-radius: var(--radius-sm);
}

/* ── Stats ── */
.stats-row {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: var(--space-3);
  margin-bottom: var(--space-6);
}

.stat-card {
  padding: var(--space-4);
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-lg);
  text-align: center;
}

.stat-value {
  display: block;
  font-size: var(--text-xl);
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: var(--space-1);
}

.stat-value.status-online { color: var(--status-online); }

.stat-label {
  font-size: var(--text-xs);
  color: var(--text-muted);
  text-transform: uppercase;
  letter-spacing: var(--tracking-wide);
}

/* ── Bottom Row ── */
.bottom-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: var(--space-4);
}

.section-panel {
  background: var(--bg-surface);
  border: 1px solid var(--border-default);
  border-radius: var(--radius-xl);
  overflow: hidden;
}

.section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-4) var(--space-5);
  border-bottom: 1px solid var(--border-subtle);
}

.section-title {
  font-size: var(--text-md);
  font-weight: 600;
  color: var(--text-primary);
}

/* ── Recent Projects ── */
.recent-projects {
  padding: var(--space-2);
}

.recent-project-item {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-3) var(--space-3);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: background var(--duration-micro);
}

.recent-project-item:hover {
  background: var(--bg-surface-hover);
}

.rp-icon { font-size: 18px; flex-shrink: 0; }

.rp-info {
  flex: 1;
  min-width: 0;
}

.rp-name {
  display: block;
  font-size: var(--text-base);
  font-weight: 500;
  color: var(--text-primary);
}

.rp-meta {
  font-size: var(--text-xs);
  color: var(--text-muted);
}

.rp-time {
  font-size: var(--text-xs);
  color: var(--text-muted);
  flex-shrink: 0;
}

/* ── Recent Logs ── */
.recent-logs {
  padding: var(--space-2);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  line-height: var(--leading-relaxed);
}

.log-line {
  display: flex;
  gap: var(--space-2);
  padding: 3px var(--space-3);
  border-radius: 3px;
}

.log-line.log-error { background: var(--status-error-bg); }

.log-time { color: var(--text-muted); flex-shrink: 0; }
.log-level { font-weight: 600; flex-shrink: 0; width: 44px; }
.log-line.log-info .log-level { color: var(--status-info); }
.log-line.log-warn .log-level { color: var(--status-warning); }
.log-line.log-error .log-level { color: var(--status-error); }
.log-line.log-debug .log-level { color: var(--text-muted); }
.log-source { color: var(--text-muted); flex-shrink: 0; }
.log-msg { color: var(--text-secondary); flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

/* ── Buttons ── */
.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-2);
  height: 32px;
  padding: 0 var(--space-4);
  border: 1px solid var(--border-default);
  background: transparent;
  color: var(--text-primary);
  font-family: var(--font-ui);
  font-size: var(--text-base);
  font-weight: 500;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--duration-micro) var(--ease-out);
}

.btn:hover { background: var(--bg-surface-hover); border-color: var(--border-strong); }
.btn:active { transform: scale(0.97); }

.btn-primary { background: var(--accent-primary); border-color: var(--accent-primary); color: white; }
.btn-primary:hover { background: var(--accent-primary-hover); border-color: var(--accent-primary-hover); }

.btn-ghost { border-color: transparent; color: var(--text-secondary); }
.btn-ghost:hover { color: var(--text-primary); background: var(--bg-surface-hover); }

.btn-sm { height: 28px; padding: 0 var(--space-3); font-size: var(--text-sm); }

/* ── Empty State ── */
.empty-placeholder {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-8);
}

.empty-placeholder p {
  font-size: var(--text-sm);
  color: var(--text-muted);
}
</style>
