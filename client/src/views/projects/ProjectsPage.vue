<template>
  <section class="projects-page">
    <header class="projects-hero">
      <div>
        <p>Project Library</p>
        <h1>Projects</h1>
        <span>{{ projects.length }} 个项目 · {{ runningTunnelCount }} 个 Tunnel 运行中 · {{ totalTunnels }} 个 Tunnel 配置</span>
      </div>
      <GButton variant="primary" icon="plus" @click="openCreate">新建项目</GButton>
    </header>

    <ProjectLoading v-if="isLoading" :count="8" />

    <GCard v-else-if="isError" variant="plain" padding="lg">
      <GErrorState title="加载失败" :message="error || '无法加载项目列表。'" retry @retry="retry" />
    </GCard>

    <div v-else-if="!hasProjects" class="project-empty-state">
      <div class="empty-illustration">
        <GIcon name="projects" :size="34" />
      </div>
      <h2>暂无项目</h2>
      <p>项目用于组织一组 Tunnel、服务器和运行状态。</p>
      <GButton variant="primary" icon="plus" @click="openCreate">创建第一个项目</GButton>
    </div>

    <template v-else>
      <div class="project-summary">
        <article>
          <span class="is-running" />
          <div><strong>{{ runningProjects.length }}</strong><small>活跃项目</small></div>
        </article>
        <article>
          <span class="is-partial" />
          <div><strong>{{ favoriteProjects.length }}</strong><small>收藏</small></div>
        </article>
        <article>
          <span class="is-stopped" />
          <div><strong>{{ recentProjects.length }}</strong><small>最近使用</small></div>
        </article>
      </div>

      <div class="project-controls">
        <label class="project-search">
          <GIcon name="search" :size="15" />
          <input v-model.trim="query" placeholder="搜索项目、标签、服务器" />
        </label>
        <select v-model="statusFilter">
          <option value="all">全部状态</option>
          <option value="running">运行中</option>
          <option value="stopped">已停止</option>
          <option value="favorite">收藏</option>
          <option value="recent">最近使用</option>
        </select>
        <select v-model="sortBy">
          <option value="lastUsedAt">最近使用</option>
          <option value="updatedAt">最近更新</option>
          <option value="name">名称</option>
          <option value="status">状态</option>
          <option value="tunnelCount">Tunnel 数量</option>
          <option value="runningTunnelCount">运行数量</option>
        </select>
      </div>

      <div class="tag-filter" aria-label="Project tags">
        <button type="button" :class="{ active: activeTag === 'all' }" @click="activeTag = 'all'">All</button>
        <button
          v-for="tag in tags"
          :key="tag"
          type="button"
          :class="{ active: activeTag === tag }"
          @click="activeTag = tag"
        >
          {{ tag }}
        </button>
      </div>

      <div v-if="finalProjects.length" class="project-grid">
        <article
          v-for="project in finalProjects"
          :key="project.id"
          class="project-card"
          :class="`is-${project.status}`"
        >
          <button type="button" class="project-card__open" @click="openProject(project.id)">
            <span class="project-card__icon"><GIcon :name="project.icon" :size="20" /></span>
            <div>
              <strong>{{ project.name }}</strong>
              <small>{{ project.serverName }} · {{ relativeTime(project.lastUsedAt) }}</small>
            </div>
          </button>

          <p>{{ project.description || "No description" }}</p>

          <div class="project-card__tags">
            <span v-for="tag in project.tags.slice(0, 3)" :key="tag">{{ tag }}</span>
            <span v-if="project.tags.length > 3">+{{ project.tags.length - 3 }}</span>
          </div>

          <div class="project-card__stats">
            <div><strong>{{ project.runningTunnelCount }}/{{ project.tunnelCount }}</strong><span>Running</span></div>
            <div><strong>{{ formatBytes(project.statistics.todayTraffic) }}</strong><span>Today</span></div>
            <div><strong>{{ project.statistics.connections }}</strong><span>Conn</span></div>
          </div>

          <footer>
            <span class="project-status"><i />{{ statusLabel(project.status) }}</span>
            <div>
              <button type="button" class="icon-action" :class="{ active: project.favorite }" @click="toggleFavorite(project.id)">
                <GIcon name="star" :size="15" />
              </button>
              <button type="button" class="icon-action" @click="openEdit(project)">
                <GIcon name="edit" :size="15" />
              </button>
              <button
                v-if="project.status === 'stopped'"
                type="button"
                class="icon-action"
                @click="startProject(project)"
              >
                <GIcon name="play" :size="15" />
              </button>
              <button v-else type="button" class="icon-action" @click="stopProject(project)">
                <GIcon name="pause" :size="15" />
              </button>
            </div>
          </footer>
        </article>
      </div>

      <div v-else class="project-empty-state is-compact">
        <div class="empty-illustration">
          <GIcon name="search" :size="30" />
        </div>
        <h2>没有匹配的项目</h2>
        <p>试试调整搜索、标签或排序条件。</p>
      </div>
    </template>

    <ProjectDialog
      v-model:visible="dialogVisible"
      :project="editingProject"
      :server-names="serverNames"
      @submit="handleSubmit"
    />
  </section>
</template>

<script setup lang="ts">
import { computed, ref } from "vue"
import { useRouter } from "vue-router"
import { useFeedback } from "@composables/useFeedback"
import GButton from "@components/base/GButton.vue"
import GCard from "@components/base/GCard.vue"
import GIcon from "@components/icons/GIcon.vue"
import GErrorState from "@components/feedback/GErrorState.vue"
import ProjectDialog from "./components/ProjectDialog.vue"
import ProjectLoading from "./components/ProjectLoading.vue"
import { useProject } from "./composables/useProject"
import type { Project, ProjectFilterType, ProjectFormData, ProjectSortType, ProjectStatus } from "./types"
import "./styles/project.css"

type SortOption = ProjectSortType | "lastUsedAt" | "runningTunnelCount"

const router = useRouter()
const { toast } = useFeedback()
const {
  projects,
  isLoading,
  isError,
  error,
  hasProjects,
  serverNames,
  runningProjects,
  favoriteProjects,
  recentProjects,
  totalTunnels,
  runningTunnelCount,
  retry,
  create,
  update,
  start,
  stop,
  toggleFavorite,
} = useProject()

const query = ref("")
const statusFilter = ref<ProjectFilterType>("all")
const activeTag = ref("all")
const sortBy = ref<SortOption>("lastUsedAt")
const dialogVisible = ref(false)
const editingProject = ref<Project | null>(null)

const tags = computed(() =>
  Array.from(new Set(projects.value.flatMap((project) => project.tags))).sort((a, b) => a.localeCompare(b)),
)

const finalProjects = computed(() => {
  const keyword = query.value.toLowerCase()
  return projects.value
    .filter((project) => {
      const matchesStatus =
        statusFilter.value === "all" ||
        (statusFilter.value === "running" && project.status !== "stopped") ||
        (statusFilter.value === "stopped" && project.status === "stopped") ||
        (statusFilter.value === "favorite" && project.favorite) ||
        statusFilter.value === "recent"
      const matchesTag = activeTag.value === "all" || project.tags.includes(activeTag.value)
      const matchesQuery =
        !keyword ||
        [project.name, project.description, project.serverName, ...project.tags]
          .join(" ")
          .toLowerCase()
          .includes(keyword)
      return matchesStatus && matchesTag && matchesQuery
    })
    .sort((a, b) => {
      if (sortBy.value === "name") return a.name.localeCompare(b.name)
      if (sortBy.value === "status") return statusOrder(a.status) - statusOrder(b.status)
      if (sortBy.value === "tunnelCount") return b.tunnelCount - a.tunnelCount
      if (sortBy.value === "runningTunnelCount") return b.runningTunnelCount - a.runningTunnelCount
      if (sortBy.value === "updatedAt") return new Date(b.updatedAt).getTime() - new Date(a.updatedAt).getTime()
      return b.lastUsedAt - a.lastUsedAt
    })
    .slice(0, statusFilter.value === "recent" ? 8 : projects.value.length)
})

function openCreate() {
  editingProject.value = null
  dialogVisible.value = true
}

function openEdit(project: Project) {
  editingProject.value = project
  dialogVisible.value = true
}

function openProject(projectId: string) {
  void router.push(`/projects/${projectId}`)
}

function startProject(project: Project) {
  start(project.id)
  toast.success(`正在启动项目「${project.name}」`)
}

function stopProject(project: Project) {
  stop(project.id)
  toast.warning(`已停止项目「${project.name}」`)
}

function handleSubmit(form: ProjectFormData, isEdit: boolean) {
  if (isEdit && editingProject.value) {
    update(editingProject.value.id, form)
    toast.success(`项目「${form.name}」已更新`)
  } else {
    create(form)
    toast.success(`项目「${form.name}」已创建`)
  }
  editingProject.value = null
}

function statusLabel(status: ProjectStatus) {
  const labels: Record<ProjectStatus, string> = {
    running: "运行中",
    partial: "部分运行",
    stopped: "已停止",
    starting: "启动中",
    error: "异常",
  }
  return labels[status]
}

function statusOrder(status: ProjectStatus) {
  const order: Record<ProjectStatus, number> = {
    running: 0,
    partial: 1,
    starting: 2,
    error: 3,
    stopped: 4,
  }
  return order[status]
}

function formatBytes(bytes: number) {
  if (!Number.isFinite(bytes) || bytes <= 0) return "0 B"
  const units = ["B", "KB", "MB", "GB", "TB"]
  const index = Math.min(units.length - 1, Math.floor(Math.log(bytes) / Math.log(1024)))
  const value = bytes / 1024 ** index
  return `${value.toFixed(value >= 10 || index === 0 ? 0 : 1)} ${units[index]}`
}

function relativeTime(timestamp: number) {
  const diff = Date.now() - timestamp
  const minutes = Math.max(1, Math.floor(diff / 60000))
  if (minutes < 60) return `${minutes} 分钟前`
  const hours = Math.floor(minutes / 60)
  if (hours < 24) return `${hours} 小时前`
  return `${Math.floor(hours / 24)} 天前`
}
</script>

<style scoped>
.projects-page {
  width: min(100%, var(--content-max-width));
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.projects-hero {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-4);
}

.projects-hero p {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
  text-transform: uppercase;
}

.projects-hero h1 {
  margin-top: 2px;
  font-size: var(--text-3xl);
  font-weight: var(--weight-semibold);
  letter-spacing: 0;
}

.projects-hero span {
  display: block;
  margin-top: var(--space-1);
  color: var(--text-secondary);
}

.project-summary {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: var(--space-3);
}

.project-summary article {
  min-height: 76px;
  display: flex;
  align-items: center;
  gap: var(--space-3);
  padding: var(--space-4);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
}

.project-summary article > span {
  width: 9px;
  height: 36px;
  border-radius: var(--radius-full);
}

.project-summary strong {
  display: block;
  color: var(--text-primary);
  font-size: var(--text-xl);
}

.project-summary small {
  color: var(--text-tertiary);
}

.project-controls {
  display: grid;
  grid-template-columns: minmax(260px, 1fr) 150px 170px;
  gap: var(--space-2);
}

.project-search,
.project-controls select {
  height: 36px;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  background: var(--bg-input);
  color: var(--text-primary);
}

.project-search {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: 0 var(--space-3);
  color: var(--text-tertiary);
}

.project-search:focus-within {
  border-color: var(--color-primary);
  box-shadow: var(--shadow-focus);
}

.project-search input {
  min-width: 0;
  flex: 1;
  border: 0;
  outline: 0;
  background: transparent;
  color: var(--text-primary);
}

.project-controls select {
  padding: 0 var(--space-3);
}

.tag-filter {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2);
}

.tag-filter button {
  height: 28px;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-full);
  background: var(--bg-surface);
  color: var(--text-secondary);
  padding: 0 var(--space-3);
  cursor: pointer;
}

.tag-filter button:hover,
.tag-filter button.active {
  border-color: var(--color-primary);
  background: var(--color-primary-muted);
  color: var(--text-primary);
}

.project-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: var(--space-4);
}

.project-card {
  position: relative;
  min-height: 252px;
  display: flex;
  flex-direction: column;
  gap: var(--space-3);
  padding: var(--space-4);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
  overflow: hidden;
}

.project-card::before {
  content: "";
  position: absolute;
  inset: 0 auto 0 0;
  width: 3px;
  background: var(--status-offline);
}

.project-card.is-running::before { background: var(--color-success); }
.project-card.is-partial::before,
.project-card.is-starting::before { background: var(--color-warning); }
.project-card.is-error::before { background: var(--color-error); }

.project-card__open {
  min-width: 0;
  display: flex;
  align-items: center;
  gap: var(--space-3);
  border: 0;
  background: transparent;
  color: inherit;
  text-align: left;
  cursor: pointer;
}

.project-card__icon {
  width: 40px;
  height: 40px;
  display: grid;
  place-items: center;
  border-radius: var(--radius-md);
  background: var(--color-primary-muted);
  color: var(--color-primary);
  flex-shrink: 0;
}

.project-card__open div {
  min-width: 0;
}

.project-card__open strong {
  display: block;
  overflow: hidden;
  color: var(--text-primary);
  font-size: var(--text-lg);
  font-weight: var(--weight-semibold);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.project-card__open small,
.project-card p {
  color: var(--text-secondary);
}

.project-card p {
  min-height: 42px;
  display: -webkit-box;
  overflow: hidden;
  line-height: var(--leading-relaxed);
  -webkit-box-orient: vertical;
  -webkit-line-clamp: 2;
}

.project-card__tags {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2);
}

.project-card__tags span {
  min-height: 24px;
  display: inline-flex;
  align-items: center;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-full);
  background: var(--bg-input);
  color: var(--text-secondary);
  padding: 0 var(--space-2);
  font-size: var(--text-xs);
}

.project-card__stats {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: var(--space-2);
  margin-top: auto;
}

.project-card__stats div {
  min-width: 0;
  padding: var(--space-2);
  border-radius: var(--radius-sm);
  background: var(--bg-input);
}

.project-card__stats strong,
.project-card__stats span {
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.project-card__stats strong {
  color: var(--text-primary);
  font-size: var(--text-sm);
}

.project-card__stats span {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

.project-card footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
}

.project-status {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  color: var(--text-secondary);
  font-size: var(--text-xs);
}

.project-status i,
.is-running,
.is-partial,
.is-stopped {
  width: 8px;
  height: 8px;
  border-radius: var(--radius-full);
  background: var(--status-offline);
}

.is-running { background: var(--color-success); }
.is-partial { background: var(--color-warning); }
.is-stopped { background: var(--status-offline); }

.project-card footer > div {
  display: flex;
  gap: var(--space-1);
}

.icon-action {
  width: 30px;
  height: 30px;
  display: grid;
  place-items: center;
  border: 1px solid transparent;
  border-radius: var(--radius-md);
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
}

.icon-action:hover,
.icon-action.active {
  border-color: var(--border-default);
  background: var(--bg-input);
  color: var(--color-primary);
}

.project-empty-state {
  min-height: 430px;
  display: grid;
  place-content: center;
  justify-items: center;
  gap: var(--space-3);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-xl);
  background: var(--bg-surface);
  color: var(--text-secondary);
  text-align: center;
}

.project-empty-state.is-compact {
  min-height: 260px;
}

.empty-illustration {
  width: 82px;
  height: 82px;
  display: grid;
  place-items: center;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-2xl);
  background: var(--color-primary-muted);
  color: var(--color-primary);
}

.project-empty-state h2 {
  color: var(--text-primary);
  font-size: var(--text-2xl);
  letter-spacing: 0;
}

.project-empty-state p {
  max-width: 420px;
}

@media (max-width: 860px) {
  .projects-hero {
    flex-direction: column;
  }

  .project-summary,
  .project-controls {
    grid-template-columns: 1fr;
  }
}
</style>
