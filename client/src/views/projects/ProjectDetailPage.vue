<!--
  ProjectDetailPage — 项目详情页
  ------------------------------------------------------------------
  现代桌面应用布局：
    Header
    ┌─────────────────────┬──────────────┐
    │ Tunnel List         │ Inspector    │
    │ Statistics          │（实时更新）  │
    └─────────────────────┴──────────────┘
  整体不是 Tab，而是左右分栏。
-->
<template>
  <div class="project-detail-page">
    <!-- 加载态 -->
    <ProjectLoading v-if="isLoading" :count="4" />

    <!-- 未找到 -->
    <GEmptyState
      v-else-if="!project"
      title="项目不存在"
      description="该项目可能已被删除或链接有误"
    >
      <template #icon>
        <GIcon name="alert-circle" :size="32" />
      </template>
      <template #action>
        <GButton variant="primary" icon="arrow-left" @click="$router.push('/projects')">
          返回项目列表
        </GButton>
      </template>
    </GEmptyState>

    <template v-else>
      <!-- 头部 -->
      <ProjectHeader
        :project="project"
        @back="$router.push('/projects')"
        @start-all="startAll"
        @stop-all="stopAll"
        @create-tunnel="onCreateTunnel"
        @edit="openEdit"
        @more="onMore"
        @toggle-pin="togglePin"
        @toggle-favorite="toggleFavorite"
      />

      <!-- 主体：左 Tunnel + 统计，右 Inspector -->
      <div class="project-detail-body">
        <main class="project-detail-main">
          <!-- Tunnel 区 -->
          <section class="project-fade-in">
            <div class="project-section__head">
              <div class="project-section__title">
                <GIcon name="router" :size="16" class="project-section__title-icon" />
                <span>Tunnel 列表</span>
                <GBadge variant="neutral" type="soft" size="sm">
                  {{ tunnels.length }}
                </GBadge>
              </div>
              <GButton size="sm" variant="ghost" icon="plus" @click="onCreateTunnel">
                添加 Tunnel
              </GButton>
            </div>

            <div class="project-tunnel-grid">
              <TunnelCard
                v-for="tunnel in tunnels"
                :key="tunnel.id"
                :tunnel="tunnel"
                :selected="selectedTunnelId === tunnel.id"
                @select="onSelectTunnel"
              />
            </div>
          </section>

          <!-- 统计区 -->
          <ProjectStatistics :project="project" class="project-fade-in" />
        </main>

        <!-- 右侧 Inspector -->
        <aside class="project-detail-aside">
          <ProjectInspector
            :project="project"
            @start="startAll"
            @stop="stopAll"
            @edit="openEdit"
            @delete="openDelete"
          />
        </aside>
      </div>
    </template>

    <!-- 编辑对话框 -->
    <ProjectDialog
      v-model:visible="dialogVisible"
      :project="project ?? null"
      :server-names="serverNames"
      @submit="handleSubmit"
    />

    <!-- 删除对话框 -->
    <ProjectDeleteDialog
      v-model:visible="deleteDialogVisible"
      :project="project ?? null"
      @confirm="handleDelete"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from "vue"
import { useRoute, useRouter } from "vue-router"
import { useFeedback } from "@composables/useFeedback"
import GIcon from "@components/icons/GIcon.vue"
import GButton from "@components/base/GButton.vue"
import GBadge from "@components/base/GBadge.vue"
import GEmptyState from "@components/feedback/GEmptyState.vue"

import ProjectHeader from "./components/ProjectHeader.vue"
import ProjectInspector from "./components/ProjectInspector.vue"
import ProjectStatistics from "./components/ProjectStatistics.vue"
import TunnelCard from "./components/TunnelCard.vue"
import ProjectLoading from "./components/ProjectLoading.vue"
import ProjectDialog from "./components/ProjectDialog.vue"
import ProjectDeleteDialog from "./components/ProjectDeleteDialog.vue"

import { useProject } from "./composables/useProject"
import { mockTunnels } from "./mock"
import type { ProjectFormData } from "./types"

import "./styles/project.css"

const route = useRoute()
const router = useRouter()
const { toast } = useFeedback()

const {
  isLoading,
  serverNames,
  getById,
  start,
  stop,
  update,
  remove,
  togglePin,
  toggleFavorite,
} = useProject()

const projectId = computed(() => route.params.projectId as string)
const project = computed(() => getById(projectId.value))

// Mock Tunnel 数据（不开发真实 Tunnel）
const tunnels = ref(structuredClone(mockTunnels))
const selectedTunnelId = ref<string | null>(null)

// 对话框
const dialogVisible = ref(false)
const deleteDialogVisible = ref(false)

function startAll() {
  if (!project.value) return
  start(project.value.id)
  toast.success(`正在启动项目「${project.value.name}」全部 Tunnel`)
}

function stopAll() {
  if (!project.value) return
  stop(project.value.id)
  toast.warning(`已停止项目「${project.value.name}」全部 Tunnel`)
}

function onCreateTunnel() {
  toast.info("Tunnel 创建功能（预留）")
}

function onMore() {
  if (!project.value) return
  toast.info(`「${project.value.name}」更多操作（预留）`)
}

function onSelectTunnel(tunnel: { id: string }) {
  selectedTunnelId.value = selectedTunnelId.value === tunnel.id ? null : tunnel.id
}

function openEdit() {
  dialogVisible.value = true
}

function openDelete() {
  deleteDialogVisible.value = true
}

function handleSubmit(form: ProjectFormData) {
  if (!project.value) return
  update(project.value.id, form)
  toast.success(`项目「${form.name}」已更新`)
}

function handleDelete() {
  if (!project.value) return
  const name = project.value.name
  remove(project.value.id)
  toast.success(`项目「${name}」已删除`)
  router.push("/projects")
}
</script>
