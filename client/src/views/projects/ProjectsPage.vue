<!--
  ProjectsPage — 项目列表页
  ------------------------------------------------------------------
  Project First 理念的核心入口。
  Card Layout，支持搜索 / 筛选 / 排序 / 收藏 / 固定 / 创建 / 编辑 / 删除。
-->
<template>
  <div class="projects-page">
    <!-- 加载态 -->
    <ProjectLoading v-if="isLoading" :count="8" />

    <!-- 错误态 -->
    <GCard v-else-if="isError" variant="plain" padding="lg">
      <GErrorState
        title="加载失败"
        :message="error || '无法加载项目列表，请重试。'"
        retry
        @retry="retry"
      />
    </GCard>

    <template v-else>
      <!-- 空状态 -->
      <ProjectEmpty v-if="!hasProjects" @create="openCreate" />

      <template v-else>
        <!-- 页面标题 -->
        <header class="projects-page__header">
          <div>
            <h1 class="projects-page__title">项目</h1>
            <p class="projects-page__subtitle">
              管理你的项目集合 · {{ projects.length }} 个项目 · {{ runningTunnelCount }} 个运行中
            </p>
          </div>
        </header>

        <!-- 工具栏 -->
        <ProjectToolbar
          :query="query"
          :filter="filter"
          :sort-by="sortBy"
          :direction="direction"
          :counts="counts"
          @update:query="query = $event"
          @update:filter="filter = $event"
          @update:sort-by="sortBy = $event"
          @update:direction="direction = $event"
          @create="openCreate"
        />

        <!-- 结果计数 -->
        <div class="projects-result-count">
          共 {{ finalProjects.length }} 个项目
          <template v-if="hasQuery">（搜索 " {{ query }} " 匹配 {{ matchCount }} 项）</template>
        </div>

        <!-- 空搜索结果 -->
        <GEmptyState
          v-if="finalProjects.length === 0"
          title="未找到匹配的项目"
          description="尝试调整搜索关键词或筛选条件"
        >
          <template #icon>
            <GIcon name="search" :size="32" />
          </template>
        </GEmptyState>

        <!-- 项目网格 -->
        <ProjectGrid v-else>
          <ProjectCard
            v-for="(project, i) in finalProjects"
            :key="project.id"
            :project="project"
            class="project-fade-in"
            :style="{ animationDelay: `${Math.min(i, 10) * 30}ms` }"
            @open="openProject"
            @edit="openEdit"
            @start="startProject"
            @stop="stopProject"
            @more="openMoreMenu"
            @toggle-pin="togglePin"
            @toggle-favorite="toggleFavorite"
            @contextmenu="openMoreMenu"
          />
        </ProjectGrid>
      </template>
    </template>

    <!-- 创建/编辑对话框 -->
    <ProjectDialog
      v-model:visible="dialogVisible"
      :project="editingProject"
      :server-names="serverNames"
      @submit="handleSubmit"
    />

    <!-- 删除确认对话框 -->
    <ProjectDeleteDialog
      v-model:visible="deleteDialogVisible"
      :project="deletingProject"
      @confirm="handleDelete"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from "vue"
import { useRouter } from "vue-router"
import { useFeedback } from "@composables/useFeedback"
import GCard from "@components/base/GCard.vue"
import GIcon from "@components/icons/GIcon.vue"
import GErrorState from "@components/feedback/GErrorState.vue"
import GEmptyState from "@components/feedback/GEmptyState.vue"

import ProjectToolbar from "./components/ProjectToolbar.vue"
import ProjectGrid from "./components/ProjectGrid.vue"
import ProjectCard from "./components/ProjectCard.vue"
import ProjectEmpty from "./components/ProjectEmpty.vue"
import ProjectLoading from "./components/ProjectLoading.vue"
import ProjectDialog from "./components/ProjectDialog.vue"
import ProjectDeleteDialog from "./components/ProjectDeleteDialog.vue"

import { useProject } from "./composables/useProject"
import { useProjectFilter } from "./composables/useProjectFilter"
import { useProjectSearch } from "./composables/useProjectSearch"
import { useProjectSort } from "./composables/useProjectSort"
import type { Project, ProjectFilterType, ProjectSortType, SortDirection, ProjectFormData } from "./types"

import "./styles/project.css"

const router = useRouter()
const { toast } = useFeedback()

const {
  projects,
  isLoading,
  isError,
  error,
  hasProjects,
  runningTunnelCount,
  serverNames,
  retry,
  create,
  update,
  remove,
  start,
  stop,
  togglePin,
  toggleFavorite,
} = useProject()

// ── 工具栏状态 ──
const query = ref("")
const filter = ref<ProjectFilterType>("all")
const sortBy = ref<ProjectSortType>("updatedAt")
const direction = ref<SortDirection>("desc")

// ── 筛选 → 搜索 → 排序 链式处理 ──
const { filtered, counts } = useProjectFilter(projects, filter)
const { results, hasQuery, matchCount } = useProjectSearch(filtered, query)
const { sorted } = useProjectSort(results, sortBy, direction)

const finalProjects = computed(() => sorted.value)

// ── 对话框状态 ──
const dialogVisible = ref(false)
const editingProject = ref<Project | null>(null)
const deleteDialogVisible = ref(false)
const deletingProject = ref<Project | null>(null)

// ── 操作处理 ──
function openCreate() {
  editingProject.value = null
  dialogVisible.value = true
}

function openEdit(project: Project) {
  editingProject.value = project
  dialogVisible.value = true
}

function openMoreMenu(project: Project) {
  // 预留：后续接入右键菜单 / 下拉菜单
  toast.info(`「${project.name}」更多操作菜单（预留）`)
}

function openProject(project: Project) {
  router.push(`/projects/${project.id}`)
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

function handleDelete(project: Project) {
  remove(project.id)
  toast.success(`项目「${project.name}」已删除`)
  deletingProject.value = null
}
</script>
