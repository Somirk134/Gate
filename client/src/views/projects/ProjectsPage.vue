<template>
  <div class="projects-page">
    <div class="page-header">
      <h1 class="page-title">Projects</h1>
      <button class="btn btn-primary" @click="showCreateDialog = true">+ New Project</button>
    </div>

    <div class="page-toolbar">
      <div class="search-box">
        <svg viewBox="0 0 18 18" fill="none" stroke="currentColor" stroke-width="1.5"><circle cx="8" cy="8" r="5.5"/><line x1="12.5" y1="12.5" x2="16" y2="16"/></svg>
        <input v-model="searchQuery" type="text" placeholder="Search projects..." class="search-input" />
      </div>
      <select v-model="sortBy" class="select-input">
        <option value="recent">Sort: Recent</option>
        <option value="name">Sort: Name</option>
        <option value="tunnels">Sort: Most Tunnels</option>
      </select>
    </div>

    <!-- Project Grid -->
    <div v-if="filteredProjects.length" class="project-grid">
      <div
        v-for="project in filteredProjects"
        :key="project.id"
        class="project-card"
        @click="$router.push(`/projects/${project.id}`)"
      >
        <div class="pc-header">
          <span class="pc-icon">{{ project.icon }}</span>
          <button class="pc-more" @click.stop>
            <svg viewBox="0 0 16 16" fill="currentColor"><circle cx="3" cy="8" r="1.5"/><circle cx="8" cy="8" r="1.5"/><circle cx="13" cy="8" r="1.5"/></svg>
          </button>
        </div>
        <h3 class="pc-name">{{ project.name }}</h3>
        <p class="pc-desc">{{ project.description }}</p>
        <div class="pc-stats">
          <div class="pc-stat">
            <span class="pc-stat-value">{{ project.tunnelCount }}</span>
            <span class="pc-stat-label">Tunnels</span>
          </div>
          <div class="pc-stat">
            <span class="pc-stat-value status-online">{{ project.onlineCount }}</span>
            <span class="pc-stat-label">Online</span>
          </div>
        </div>
        <span class="pc-time">Started {{ project.lastActive }}</span>
      </div>
    </div>

    <!-- Empty State -->
    <div v-else class="empty-state">
      <div class="empty-icon">
        <svg viewBox="0 0 48 48" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M6 8h18l6 10h16v20a4 4 0 0 1-4 4H10a4 4 0 0 1-4-4V12a4 4 0 0 1 4-4z"/></svg>
      </div>
      <h2 class="empty-title">No projects yet</h2>
      <p class="empty-desc">Create your first project to start managing tunnels</p>
      <button class="btn btn-primary" @click="showCreateDialog = true">Create your first project</button>
    </div>

    <!-- Create Dialog (placeholder) -->
    <div v-if="showCreateDialog" class="modal-overlay" @click.self="showCreateDialog = false">
      <div class="modal">
        <div class="modal-header">
          <h3>New Project</h3>
          <button class="btn-icon" @click="showCreateDialog = false">
            <svg viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.5"><path d="M4 4l8 8M12 4l-8 8"/></svg>
          </button>
        </div>
        <div class="modal-body">
          <label class="form-label">Name</label>
          <input v-model="newProjectName" type="text" class="form-input" placeholder="My API Service" />
          <label class="form-label">Description</label>
          <input v-model="newProjectDesc" type="text" class="form-input" placeholder="Backend microservices" />
        </div>
        <div class="modal-footer">
          <button class="btn btn-secondary" @click="showCreateDialog = false">Cancel</button>
          <button class="btn btn-primary" @click="createProject">Create Project</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'

const searchQuery = ref('')
const sortBy = ref('recent')
const showCreateDialog = ref(false)
const newProjectName = ref('')
const newProjectDesc = ref('')

const projects = ref([
  { id: '1', icon: '📦', name: 'My API Service', description: 'Backend microservices and APIs', tunnelCount: 8, onlineCount: 3, lastActive: '2 hours ago' },
  { id: '2', icon: '🌐', name: 'Web App Frontend', description: 'Public-facing web applications', tunnelCount: 3, onlineCount: 1, lastActive: '1 day ago' },
  { id: '3', icon: '🛠', name: 'Dev Environment', description: 'Local development tools and services', tunnelCount: 5, onlineCount: 5, lastActive: 'Just now' },
  { id: '4', icon: '📊', name: 'Monitoring Stack', description: 'Grafana, Prometheus, and alerting', tunnelCount: 4, onlineCount: 2, lastActive: '3 hours ago' },
  { id: '5', icon: '🔌', name: 'IoT Gateway', description: 'IoT device management and control', tunnelCount: 2, onlineCount: 0, lastActive: '5 days ago' },
])

const filteredProjects = computed(() => {
  let result = projects.value
  if (searchQuery.value) {
    const q = searchQuery.value.toLowerCase()
    result = result.filter(p => p.name.toLowerCase().includes(q) || p.description.toLowerCase().includes(q))
  }
  if (sortBy.value === 'name') result = [...result].sort((a, b) => a.name.localeCompare(b.name))
  if (sortBy.value === 'tunnels') result = [...result].sort((a, b) => b.tunnelCount - a.tunnelCount)
  return result
})

function createProject() {
  if (newProjectName.value.trim()) {
    projects.value.unshift({
      id: String(Date.now()),
      icon: '📁',
      name: newProjectName.value.trim(),
      description: newProjectDesc.value.trim() || 'No description',
      tunnelCount: 0,
      onlineCount: 0,
      lastActive: 'Just now',
    })
    showCreateDialog.value = false
    newProjectName.value = ''
    newProjectDesc.value = ''
  }
}
</script>

<style scoped>
.projects-page { max-width: 1024px; margin: 0 auto; }

.page-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: var(--space-5); }
.page-title { font-size: var(--text-xl); font-weight: 600; color: var(--text-primary); }

.page-toolbar { display: flex; align-items: center; gap: var(--space-3); margin-bottom: var(--space-5); }

.search-box { display: flex; align-items: center; gap: var(--space-2); flex: 1; max-width: 320px; padding: 0 var(--space-3); height: 32px; background: var(--bg-input); border: 1px solid var(--border-default); border-radius: var(--radius-md); }
.search-box svg { width: 14px; height: 14px; color: var(--text-muted); flex-shrink: 0; }
.search-input { flex: 1; border: none; background: transparent; color: var(--text-primary); font-family: var(--font-ui); font-size: var(--text-base); outline: none; }
.search-input::placeholder { color: var(--text-muted); }

.select-input { height: 32px; padding: 0 var(--space-3); background: var(--bg-input); border: 1px solid var(--border-default); border-radius: var(--radius-md); color: var(--text-secondary); font-family: var(--font-ui); font-size: var(--text-sm); cursor: pointer; }

/* ── Project Grid ── */
.project-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(280px, 1fr)); gap: var(--space-4); }

.project-card { padding: var(--space-5); background: var(--bg-surface); border: 1px solid var(--border-default); border-radius: var(--radius-xl); cursor: pointer; transition: all var(--duration-standard) var(--ease-out); }
.project-card:hover { border-color: var(--border-strong); background: var(--bg-surface-hover); transform: translateY(-2px); box-shadow: var(--shadow-sm); }

.pc-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: var(--space-3); }
.pc-icon { font-size: 24px; }
.pc-more { width: 28px; height: 28px; display: flex; align-items: center; justify-content: center; border: none; background: transparent; color: var(--text-muted); border-radius: var(--radius-md); cursor: pointer; }
.pc-more:hover { background: var(--bg-surface-hover); color: var(--text-secondary); }
.pc-more svg { width: 14px; height: 14px; }

.pc-name { font-size: var(--text-lg); font-weight: 600; color: var(--text-primary); margin-bottom: var(--space-1); }
.pc-desc { font-size: var(--text-base); color: var(--text-secondary); margin-bottom: var(--space-4); line-height: var(--leading-normal); }

.pc-stats { display: flex; gap: var(--space-2); margin-bottom: var(--space-3); }
.pc-stat { flex: 1; padding: var(--space-2); background: var(--bg-input); border-radius: var(--radius-md); text-align: center; }
.pc-stat-value { display: block; font-size: var(--text-md); font-weight: 600; color: var(--text-primary); }
.pc-stat-value.status-online { color: var(--status-online); }
.pc-stat-label { font-size: var(--text-xs); color: var(--text-muted); }
.pc-time { font-size: var(--text-xs); color: var(--text-muted); }

/* ── Buttons ── */
.btn { display: inline-flex; align-items: center; gap: var(--space-2); height: 32px; padding: 0 var(--space-4); border: 1px solid var(--border-default); background: transparent; color: var(--text-primary); font-family: var(--font-ui); font-size: var(--text-base); font-weight: 500; border-radius: var(--radius-md); cursor: pointer; transition: all var(--duration-micro) var(--ease-out); }
.btn:hover { background: var(--bg-surface-hover); border-color: var(--border-strong); }
.btn:active { transform: scale(0.97); }
.btn-primary { background: var(--accent-primary); border-color: var(--accent-primary); color: white; }
.btn-primary:hover { background: var(--accent-primary-hover); border-color: var(--accent-primary-hover); }
.btn-secondary { background: transparent; border-color: var(--border-default); color: var(--text-primary); }
.btn-secondary:hover { background: var(--bg-surface-hover); }

/* ── Modal ── */
.modal-overlay { position: fixed; inset: 0; background: var(--overlay); display: flex; align-items: center; justify-content: center; z-index: 100; backdrop-filter: blur(4px); }
.modal { width: 440px; background: var(--bg-surface-raised); border: 1px solid var(--border-default); border-radius: var(--radius-2xl); box-shadow: var(--shadow-lg); animation: modalIn var(--duration-entrance) var(--ease-out); }
@keyframes modalIn { from { opacity: 0; transform: scale(0.96); } to { opacity: 1; transform: scale(1); } }
.modal-header { display: flex; align-items: center; justify-content: space-between; padding: var(--space-5) var(--space-5) var(--space-3); }
.modal-header h3 { font-size: var(--text-lg); font-weight: 600; }
.modal-body { padding: 0 var(--space-5) var(--space-5); display: flex; flex-direction: column; gap: var(--space-3); }
.modal-footer { display: flex; justify-content: flex-end; gap: var(--space-2); padding: var(--space-3) var(--space-5) var(--space-5); }
.btn-icon { width: 28px; height: 28px; display: flex; align-items: center; justify-content: center; border: none; background: transparent; color: var(--text-muted); border-radius: var(--radius-md); cursor: pointer; }
.btn-icon:hover { background: var(--bg-surface-hover); color: var(--text-primary); }
.btn-icon svg { width: 14px; height: 14px; }

.form-label { font-size: var(--text-sm); font-weight: 500; color: var(--text-secondary); }
.form-input { height: 32px; padding: 0 var(--space-3); background: var(--bg-input); border: 1px solid var(--border-default); border-radius: var(--radius-md); color: var(--text-primary); font-family: var(--font-ui); font-size: var(--text-base); }
.form-input:focus { border-color: var(--border-accent); outline: none; box-shadow: 0 0 0 3px rgba(91,141,239,0.15); }

/* ── Empty ── */
.empty-state { display: flex; flex-direction: column; align-items: center; justify-content: center; padding: var(--space-12) var(--space-6); text-align: center; }
.empty-icon { width: 64px; height: 64px; color: var(--text-muted); margin-bottom: var(--space-4); }
.empty-icon svg { width: 100%; height: 100%; }
.empty-title { font-size: var(--text-lg); font-weight: 600; color: var(--text-primary); margin-bottom: var(--space-2); }
.empty-desc { font-size: var(--text-base); color: var(--text-muted); margin-bottom: var(--space-5); }
</style>
