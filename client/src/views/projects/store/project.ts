/* ==================================================================
   Project Store
   ------------------------------------------------------------------
   集中管理 Project 模块的所有数据与状态。
   当前从 mock 加载，后续替换为真实接口时，只需将 load 动作改为
   API 调用并保持返回类型不变即可无缝迁移。

   状态机：idle → loading → success / error
   ================================================================== */

import { defineStore } from "pinia"
import { ref, computed } from "vue"
import type {
  Project,
  ProjectFormData,
  ProjectLoadStatus,
} from "../types"
import { mockProjects, mockServerNames } from "../mock"
import { genId } from "../utils"

export const useProjectStore = defineStore("project", () => {
  // === State ===
  const projects = ref<Project[]>([])
  const status = ref<ProjectLoadStatus>("idle")
  const error = ref<string>("")
  const lastUpdated = ref<number>(0)
  const serverNames = ref<string[]>([...mockServerNames])

  // === Getters ===
  const isLoading = computed(() => status.value === "loading")
  const isError = computed(() => status.value === "error")
  const isReady = computed(() => status.value === "success")
  const hasProjects = computed(() => projects.value.length > 0)

  const pinnedProjects = computed(() =>
    projects.value.filter((p) => p.pinned),
  )
  const favoriteProjects = computed(() =>
    projects.value.filter((p) => p.favorite),
  )
  const runningProjects = computed(() =>
    projects.value.filter(
      (p) => p.status === "running" || p.status === "partial" || p.status === "starting",
    ),
  )
  const stoppedProjects = computed(() =>
    projects.value.filter((p) => p.status === "stopped"),
  )
  const recentProjects = computed(() =>
    [...projects.value].sort((a, b) => b.lastUsedAt - a.lastUsedAt).slice(0, 8),
  )

  const totalTunnels = computed(() =>
    projects.value.reduce((sum, p) => sum + p.tunnelCount, 0),
  )
  const runningTunnelCount = computed(() =>
    projects.value.reduce((sum, p) => sum + p.runningTunnelCount, 0),
  )

  function getById(id: string): Project | undefined {
    return projects.value.find((p) => p.id === id)
  }

  // === Actions ===
  async function load(): Promise<void> {
    status.value = "loading"
    error.value = ""
    try {
      // 模拟网络延迟，便于展示 Loading State
      await new Promise((resolve) => setTimeout(resolve, 500))
      projects.value = structuredClone(mockProjects)
      status.value = "success"
      lastUpdated.value = Date.now()
    } catch (e) {
      status.value = "error"
      error.value = e instanceof Error ? e.message : "加载失败"
    }
  }

  async function refresh(): Promise<void> {
    return load()
  }

  /** 创建项目 */
  function createProject(form: ProjectFormData): Project {
    const now = Date.now()
    const project: Project = {
      id: genId(),
      name: form.name.trim(),
      description: form.description.trim(),
      icon: form.icon,
      color: form.color,
      tags: [...form.tags],
      serverName: form.serverName,
      autoStart: form.autoStart,
      remark: form.remark.trim(),
      status: "stopped",
      pinned: false,
      favorite: false,
      lastUsedAt: now,
      tunnelCount: 0,
      runningTunnelCount: 0,
      statistics: {
        todayTraffic: 0,
        totalTraffic: 0,
        uptime: 0,
        connections: 0,
        tunnelCount: 0,
        runningTunnelCount: 0,
      },
      lastStartedAt: "—",
      createdAt: new Date(now).toISOString(),
      updatedAt: new Date(now).toISOString(),
    }
    projects.value.unshift(project)
    return project
  }

  /** 更新项目（Mock 自动保存） */
  function updateProject(id: string, patch: Partial<ProjectFormData>): void {
    const p = projects.value.find((x) => x.id === id)
    if (!p) return
    if (patch.name !== undefined) p.name = patch.name.trim()
    if (patch.description !== undefined) p.description = patch.description.trim()
    if (patch.icon !== undefined) p.icon = patch.icon
    if (patch.color !== undefined) p.color = patch.color
    if (patch.tags !== undefined) p.tags = [...patch.tags]
    if (patch.serverName !== undefined) p.serverName = patch.serverName
    if (patch.autoStart !== undefined) p.autoStart = patch.autoStart
    if (patch.remark !== undefined) p.remark = patch.remark.trim()
    p.updatedAt = new Date().toISOString()
  }

  /** 删除项目 */
  function removeProject(id: string): void {
    const idx = projects.value.findIndex((x) => x.id === id)
    if (idx !== -1) projects.value.splice(idx, 1)
  }

  /** 启动项目（启动全部 Tunnel） */
  function startProject(id: string): void {
    const p = projects.value.find((x) => x.id === id)
    if (!p) return
    p.status = "starting"
    p.lastUsedAt = Date.now()
    setTimeout(() => {
      if (!p) return
      p.status = "running"
      p.runningTunnelCount = p.tunnelCount
      p.lastStartedAt = "刚刚"
      p.statistics.runningTunnelCount = p.tunnelCount
      p.statistics.uptime = 1
    }, 1000)
  }

  /** 停止项目（停止全部 Tunnel） */
  function stopProject(id: string): void {
    const p = projects.value.find((x) => x.id === id)
    if (!p) return
    p.status = "stopped"
    p.runningTunnelCount = 0
    p.lastStartedAt = "—"
    p.statistics.runningTunnelCount = 0
    p.statistics.uptime = 0
  }

  /** 切换固定 */
  function togglePin(id: string): void {
    const p = projects.value.find((x) => x.id === id)
    if (p) p.pinned = !p.pinned
  }

  /** 切换收藏 */
  function toggleFavorite(id: string): void {
    const p = projects.value.find((x) => x.id === id)
    if (p) p.favorite = !p.favorite
  }

  return {
    // state
    projects,
    status,
    error,
    lastUpdated,
    serverNames,
    // getters
    isLoading,
    isError,
    isReady,
    hasProjects,
    pinnedProjects,
    favoriteProjects,
    runningProjects,
    stoppedProjects,
    recentProjects,
    totalTunnels,
    runningTunnelCount,
    getById,
    // actions
    load,
    refresh,
    createProject,
    updateProject,
    removeProject,
    startProject,
    stopProject,
    togglePin,
    toggleFavorite,
  }
})
