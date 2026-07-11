<template>
  <section class="projects-page">
    <header class="projects-page__header">
      <div>
        <p class="projects-page__eyebrow">{{ t('project.workspace') }}</p>
        <h1 class="projects-page__title">{{ t('project.title') }}</h1>
        <p class="projects-page__subtitle">
          {{
            t('project.workspaceSummary', {
              tunnels: totalTunnels,
              running: runningTunnelCount,
              projects: projects.length,
            })
          }}
        </p>
      </div>
      <GButton variant="primary" icon="plus" @click="openCreate">
        {{ t('project.newProject') }}
      </GButton>
    </header>

    <ProjectLoading v-if="isLoading" :count="8" />

    <GCard v-else-if="isError" variant="plain" padding="lg">
      <GErrorState
        :title="t('project.loadFailed')"
        :message="error || t('project.loadFailedMessage')"
        retry
        @retry="retry" />
    </GCard>

    <ProjectEmpty v-else-if="!hasProjects" @create="openCreate" />

    <template v-else>
      <ProjectToolbar
        v-model:query="query"
        v-model:filter="filter"
        v-model:sort-by="sortBy"
        v-model:direction="direction"
        :counts="counts"
        @create="openCreate" />

      <div class="projects-result-count">
        {{ t('project.resultCount', { count: sorted.length }) }}
      </div>

      <ProjectGrid v-if="sorted.length">
        <ProjectCard
          v-for="project in sorted"
          :key="project.id"
          :project="project"
          :loading="startingId === project.id || stoppingId === project.id"
          @open="openProject"
          @edit="openEdit"
          @delete="openDelete"
          @start="startWorkspace"
          @stop="stopWorkspace"
          @toggle-pin="togglePinProject"
          @toggle-favorite="toggleFavoriteProject" />
      </ProjectGrid>

      <div v-else class="projects-no-result">
        <GIcon name="search" :size="28" />
        <span>{{ t('project.noResult') }}</span>
      </div>
    </template>

    <ProjectDialog
      v-model:visible="dialogVisible"
      :project="editingProject"
      :server-names="serverNames"
      @submit="handleSubmit" />

    <ProjectDeleteDialog
      v-model:visible="deleteVisible"
      :project="deletingProject"
      @confirm="handleDelete" />
  </section>
</template>

<script setup lang="ts">
defineOptions({ name: 'projects' })
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRouter } from 'vue-router'
import { useFeedback } from '@composables/useFeedback'
import GButton from '@components/base/GButton.vue'
import GCard from '@components/base/GCard.vue'
import GIcon from '@components/icons/GIcon.vue'
import GErrorState from '@components/feedback/GErrorState.vue'
import { reopenOverlay } from '@/utils/i18n'
import { formatTunnelOperationError } from '@/utils/operationError'
import ProjectCard from './components/ProjectCard.vue'
import ProjectDeleteDialog from './components/ProjectDeleteDialog.vue'
import ProjectDialog from './components/ProjectDialog.vue'
import ProjectEmpty from './components/ProjectEmpty.vue'
import ProjectGrid from './components/ProjectGrid.vue'
import ProjectLoading from './components/ProjectLoading.vue'
import ProjectToolbar from './components/ProjectToolbar.vue'
import { useProject } from './composables/useProject'
import { useProjectFilter } from './composables/useProjectFilter'
import { useProjectSearch } from './composables/useProjectSearch'
import { useProjectSort } from './composables/useProjectSort'
import type {
  Project,
  ProjectDeleteMode,
  ProjectFilterType,
  ProjectFormData,
  ProjectSortType,
  SortDirection,
} from './types'

const router = useRouter()
const { t } = useI18n()
const { toast, notify } = useFeedback()
const {
  projects,
  serverNames,
  isLoading,
  isError,
  error,
  hasProjects,
  totalTunnels,
  runningTunnelCount,
  retry,
  create,
  update,
  remove,
  start,
  stop,
  togglePin,
  toggleFavorite,
  startingId,
  stoppingId,
} = useProject()

const query = ref('')
const filter = ref<ProjectFilterType>('all')
const sortBy = ref<ProjectSortType>('updatedAt')
const direction = ref<SortDirection>('desc')
const dialogVisible = ref(false)
const deleteVisible = ref(false)
const editingProject = ref<Project | null>(null)
const deletingProject = ref<Project | null>(null)

const { results } = useProjectSearch(projects, query)
const { filtered, counts } = useProjectFilter(results, filter)
const { sorted } = useProjectSort(filtered, sortBy, direction)

async function openCreate() {
  editingProject.value = null
  await reopenOverlay(dialogVisible)
}

async function openEdit(project: Project) {
  editingProject.value = project
  await reopenOverlay(dialogVisible)
}

async function openDelete(project: Project) {
  deletingProject.value = project
  await reopenOverlay(deleteVisible)
}

function openProject(project: Project) {
  void router.push(`/projects/${project.id}`)
}

async function handleSubmit(form: ProjectFormData, isEdit: boolean) {
  try {
    if (isEdit && editingProject.value) {
      await update(editingProject.value.id, form)
      toast.success(t('project.notifications.updated', { name: form.name }))
      return
    }
    const created = await create(form)
    toast.success(t('project.notifications.created', { name: created.name }))
  } catch (err) {
    notify.error(t('project.notifications.saveFailed'), errorMessage(err), 10000)
  }
}

async function handleDelete(project: Project, mode: ProjectDeleteMode) {
  try {
    await remove(project.id, mode)
    toast.success(t('project.notifications.deleted', { name: project.name }))
  } catch (err) {
    notify.error(t('project.notifications.deleteFailed'), errorMessage(err), 10000)
  }
}

async function togglePinProject(id: string) {
  try {
    await togglePin(id)
  } catch (err) {
    notify.error(t('project.notifications.pinFailed'), errorMessage(err), 8000)
  }
}

async function toggleFavoriteProject(id: string) {
  try {
    await toggleFavorite(id)
  } catch (err) {
    notify.error(t('project.notifications.favoriteFailed'), errorMessage(err), 8000)
  }
}

async function startWorkspace(project: Project) {
  if (startingId.value) return
  try {
    await start(project.id)
    toast.success(t('project.notifications.started', { name: project.name }))
  } catch (err) {
    notify.error(t('project.notifications.startFailed'), errorMessage(err), 8000)
  }
}

async function stopWorkspace(project: Project) {
  if (stoppingId.value) return
  try {
    await stop(project.id)
    toast.warning(t('project.notifications.stopped', { name: project.name }))
  } catch (err) {
    notify.error(t('project.notifications.stopFailed'), errorMessage(err), 8000)
  }
}

function errorMessage(err: unknown): string {
  return formatTunnelOperationError(err, 'project.notifications.storageCheck')
}
</script>

<style src="./styles/project.css"></style>
<style scoped>
.projects-page__eyebrow {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
  text-transform: uppercase;
}

.projects-no-result {
  min-height: 280px;
  display: grid;
  place-content: center;
  justify-items: center;
  gap: var(--space-3);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
  color: var(--text-tertiary);
}
</style>
