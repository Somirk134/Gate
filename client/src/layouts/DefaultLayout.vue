<template>
  <div class="app-shell" :class="{ 'sidebar-collapsed': sidebarCollapsed, 'inspector-open': inspectorOpen }">
    <!-- Sidebar -->
    <aside class="sidebar">
      <div class="sidebar-header">
        <div class="sidebar-logo" @click="toggleSidebar">
          <div class="logo-icon">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z"/>
              <polyline points="9 22 9 12 15 12 15 22"/>
            </svg>
          </div>
          <span v-show="!sidebarCollapsed" class="logo-text">Gate</span>
        </div>
      </div>

      <nav class="sidebar-nav">
        <NavItem
          v-for="item in navItems"
          :key="item.path"
          :item="item"
          :collapsed="sidebarCollapsed"
          :active="isActive(item.path)"
          @click="navigateTo(item.path)"
        />
      </nav>

      <div class="sidebar-footer">
        <div class="connection-status" :class="connectionStatus">
          <span class="status-dot"></span>
          <span v-show="!sidebarCollapsed" class="status-text">{{ connectionLabel }}</span>
        </div>
      </div>
    </aside>

    <!-- Main Content Area -->
    <div class="main-area">
      <!-- Top Toolbar -->
      <header class="toolbar">
        <div class="toolbar-left">
          <button class="toolbar-btn" title="Back" @click="router.back()">
            <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M10 3L5 8l5 5"/></svg>
          </button>
          <button class="toolbar-btn" title="Forward" @click="router.forward()">
            <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M6 3l5 5-5 5"/></svg>
          </button>
          <div class="toolbar-breadcrumb" v-if="breadcrumbs.length">
            <template v-for="(crumb, i) in breadcrumbs" :key="i">
              <span v-if="i > 0" class="breadcrumb-sep">/</span>
              <span class="breadcrumb-item" :class="{ active: i === breadcrumbs.length - 1 }">{{ crumb }}</span>
            </template>
          </div>
        </div>
        <div class="toolbar-right">
          <button class="toolbar-btn cmd-k-badge" title="Command Palette (⌘K)">
            <span>⌘K</span>
          </button>
          <button class="toolbar-btn" title="Toggle Inspector (⌘⇧I)" @click="toggleInspector">
            <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><rect x="1" y="1" width="6" height="6" rx="1"/><rect x="9" y="1" width="6" height="6" rx="1"/><rect x="1" y="9" width="6" height="6" rx="1"/><rect x="9" y="9" width="6" height="6" rx="1"/></svg>
          </button>
        </div>
      </header>

      <!-- Content -->
      <main class="content">
        <router-view />
      </main>

      <!-- Status Bar -->
      <footer class="statusbar">
        <div class="statusbar-left">
          <span class="status-indicator" :class="connectionStatus"></span>
          <span class="status-text">{{ connectionLabel }}</span>
        </div>
        <div class="statusbar-center">
          <span class="status-stat">{{ onlineCount }} / {{ totalCount }} tunnels online</span>
        </div>
        <div class="statusbar-right">
          <span class="status-stat">↑ {{ formatBytes(uploadBytes) }}</span>
          <span class="status-stat">↓ {{ formatBytes(downloadBytes) }}</span>
        </div>
      </footer>
    </div>

    <!-- Inspector Panel (conditionally shown) -->
    <aside v-if="inspectorOpen" class="inspector">
      <div class="inspector-header">
        <span class="inspector-title">Details</span>
        <button class="toolbar-btn" @click="toggleInspector">
          <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M4 4l8 8M12 4l-8 8"/></svg>
        </button>
      </div>
      <div class="inspector-content">
        <!-- Slot for page-specific inspector content -->
        <slot name="inspector">
          <div class="inspector-placeholder">
            <p class="placeholder-text">Select an item to view details</p>
          </div>
        </slot>
      </div>
    </aside>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import NavItem from '@components/common/NavItem.vue'

const router = useRouter()
const route = useRoute()

// === Sidebar State ===
const sidebarCollapsed = ref(false)
const toggleSidebar = () => { sidebarCollapsed.value = !sidebarCollapsed.value }

// === Inspector State ===
const inspectorOpen = ref(false)
const toggleInspector = () => { inspectorOpen.value = !inspectorOpen.value }

// === Navigation Items ===
const navItems = [
  { path: '/', label: 'Dashboard', icon: 'dashboard' },
  { path: '/projects', label: 'Projects', icon: 'projects' },
  { path: '/servers', label: 'Servers', icon: 'servers' },
  { path: '/logs', label: 'Logs', icon: 'logs' },
  { path: '/settings', label: 'Settings', icon: 'settings' },
  { path: '/about', label: 'About', icon: 'about' },
]

const isActive = (path: string) => {
  if (path === '/') return route.path === '/'
  return route.path.startsWith(path)
}

const navigateTo = (path: string) => {
  router.push(path)
}

// === Breadcrumbs ===
const breadcrumbs = computed(() => {
  const parts = route.path.split('/').filter(Boolean)
  if (parts.length === 0) return ['Dashboard']
  return parts.map(p => p.charAt(0).toUpperCase() + p.slice(1).replace(/-/g, ' '))
})

// === Connection Status (placeholder — will be reactive store later) ===
const connectionStatus = ref<'online' | 'offline' | 'connecting'>('online')
const connectionLabel = computed(() => {
  switch (connectionStatus.value) {
    case 'online': return 'Connected'
    case 'offline': return 'Offline'
    case 'connecting': return 'Connecting...'
  }
})

// === Stats Placeholders ===
const onlineCount = ref(3)
const totalCount = ref(8)
const uploadBytes = ref(1024 * 1024 * 50)
const downloadBytes = ref(1024 * 1024 * 120)

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`
}

onMounted(() => {
  // Listen for keyboard shortcuts
  document.addEventListener('keydown', handleKeydown)
})

function handleKeydown(e: KeyboardEvent) {
  const mod = e.metaKey || e.ctrlKey
  if (mod && e.key === '\\') {
    e.preventDefault()
    toggleSidebar()
  }
  if (mod && e.shiftKey && e.key === 'I') {
    e.preventDefault()
    toggleInspector()
  }
}
</script>

<style scoped>
.app-shell {
  display: grid;
  grid-template-columns: var(--sidebar-width) 1fr;
  height: 100vh;
  overflow: hidden;
  transition: grid-template-columns var(--duration-standard) var(--ease-out);
}

.app-shell.sidebar-collapsed {
  grid-template-columns: var(--sidebar-collapsed-width) 1fr;
}

.app-shell.inspector-open {
  grid-template-columns: var(--sidebar-width) 1fr var(--inspector-width);
}

.app-shell.sidebar-collapsed.inspector-open {
  grid-template-columns: var(--sidebar-collapsed-width) 1fr var(--inspector-width);
}

/* ── Sidebar ── */
.sidebar {
  display: flex;
  flex-direction: column;
  background: var(--bg-sidebar);
  border-right: 1px solid var(--border-subtle);
  user-select: none;
  overflow: hidden;
}

.sidebar-header {
  padding: var(--space-4);
  border-bottom: 1px solid var(--border-subtle);
}

.sidebar-logo {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  cursor: pointer;
  padding: var(--space-1) 0;
}

.logo-icon {
  width: 24px;
  height: 24px;
  color: var(--accent-primary);
  flex-shrink: 0;
}

.logo-icon svg {
  width: 100%;
  height: 100%;
}

.logo-text {
  font-size: var(--text-md);
  font-weight: 600;
  color: var(--text-primary);
  letter-spacing: var(--tracking-tight);
}

.sidebar-nav {
  flex: 1;
  padding: var(--space-2);
  overflow-y: auto;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.sidebar-footer {
  padding: var(--space-3) var(--space-4);
  border-top: 1px solid var(--border-subtle);
}

.connection-status {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-2) 0;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: var(--radius-full);
  flex-shrink: 0;
}

.connection-status.online .status-dot { background: var(--status-online); }
.connection-status.offline .status-dot { background: var(--status-offline); }
.connection-status.connecting .status-dot { background: var(--status-warning); animation: pulse 1.5s infinite; }

@keyframes pulse {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.3; }
}

.status-text {
  font-size: var(--text-xs);
  color: var(--text-muted);
}

/* ── Main Area ── */
.main-area {
  display: flex;
  flex-direction: column;
  min-width: 0;
  overflow: hidden;
}

/* ── Toolbar ── */
.toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: var(--toolbar-height);
  padding: 0 var(--space-3);
  background: var(--bg-toolbar);
  border-bottom: 1px solid var(--border-subtle);
  flex-shrink: 0;
}

.toolbar-left,
.toolbar-right {
  display: flex;
  align-items: center;
  gap: var(--space-1);
}

.toolbar-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--duration-micro) var(--ease-out);
}

.toolbar-btn:hover {
  background: var(--bg-surface-hover);
  color: var(--text-primary);
}

.toolbar-btn svg {
  width: 14px;
  height: 14px;
}

.cmd-k-badge {
  width: auto;
  padding: 0 var(--space-2);
  font-size: var(--text-xs);
  font-weight: 500;
  color: var(--text-muted);
  border: 1px solid var(--border-default);
  letter-spacing: var(--tracking-wide);
}

.toolbar-breadcrumb {
  display: flex;
  align-items: center;
  gap: var(--space-1);
  margin-left: var(--space-3);
  font-size: var(--text-sm);
}

.breadcrumb-sep {
  color: var(--text-muted);
}

.breadcrumb-item {
  color: var(--text-muted);
  cursor: pointer;
  transition: color var(--duration-micro);
}

.breadcrumb-item:hover {
  color: var(--text-secondary);
}

.breadcrumb-item.active {
  color: var(--text-primary);
  font-weight: 500;
}

/* ── Content ── */
.content {
  flex: 1;
  overflow-y: auto;
  overflow-x: hidden;
  padding: var(--space-6);
}

/* ── Status Bar ── */
.statusbar {
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
}

.statusbar-left,
.statusbar-center,
.statusbar-right {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}

.status-indicator {
  width: 6px;
  height: 6px;
  border-radius: var(--radius-full);
  background: var(--status-online);
}

.status-indicator.offline { background: var(--status-offline); }
.status-indicator.connecting { background: var(--status-warning); }

/* ── Inspector ── */
.inspector {
  display: flex;
  flex-direction: column;
  background: var(--bg-sidebar);
  border-left: 1px solid var(--border-subtle);
  overflow: hidden;
}

.inspector-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: var(--toolbar-height);
  padding: 0 var(--space-3);
  border-bottom: 1px solid var(--border-subtle);
}

.inspector-title {
  font-size: var(--text-sm);
  font-weight: 500;
  color: var(--text-secondary);
}

.inspector-content {
  flex: 1;
  overflow-y: auto;
  padding: var(--space-4);
}

.inspector-placeholder {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
}

.placeholder-text {
  font-size: var(--text-sm);
  color: var(--text-muted);
}
</style>
