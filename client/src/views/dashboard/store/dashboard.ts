/* ==================================================================
   Dashboard Store
   ------------------------------------------------------------------
   集中管理 Dashboard 的所有数据与状态。
   当前从 mock 加载，后续替换为真实接口时，只需将 load 动作改为
   API 调用并保持返回类型不变即可无缝迁移。
   ================================================================== */

import { defineStore } from "pinia"
import { ref, computed } from "vue"
import type {
  DashboardProject,
  DashboardTunnel,
  DashboardServer,
  DashboardActivity,
  DashboardStatistics,
  DashboardResource,
  DashboardNews,
  DashboardQuickAction,
  DashboardLoadStatus,
  TunnelStatus,
} from "../types"
import {
  mockProjects,
  mockTunnels,
  mockServers,
  mockActivities,
  mockStatistics,
  mockResource,
  mockNews,
  quickActions,
  developerQuotes,
} from "../mock"

export const useDashboardStore = defineStore("dashboard", () => {
  // === State ===
  const projects = ref<DashboardProject[]>([])
  const tunnels = ref<DashboardTunnel[]>([])
  const servers = ref<DashboardServer[]>([])
  const activities = ref<DashboardActivity[]>([])
  const statistics = ref<DashboardStatistics | null>(null)
  const resource = ref<DashboardResource | null>(null)
  const news = ref<DashboardNews[]>([])
  const actions = ref<DashboardQuickAction[]>(quickActions)
  const quotes = ref<string[]>(developerQuotes)

  const status = ref<DashboardLoadStatus>("idle")
  const error = ref<string>("")
  const lastUpdated = ref<number>(0)

  // === Getters ===
  const isLoading = computed(() => status.value === "loading")
  const isError = computed(() => status.value === "error")
  const isReady = computed(() => status.value === "success")

  const pinnedProjects = computed(() =>
    projects.value.filter((p) => p.pinned),
  )
  const favoriteProjects = computed(() =>
    projects.value.filter((p) => p.favorite),
  )
  const runningTunnels = computed(() =>
    tunnels.value.filter(
      (t) => t.status === "online" || t.status === "connecting",
    ),
  )
  const onlineServers = computed(() =>
    servers.value.filter((s) => s.status === "online"),
  )
  const connectedServerCount = computed(() =>
    servers.value.filter((s) => s.connected).length,
  )
  const totalConnections = computed(() =>
    tunnels.value.reduce((sum, t) => sum + t.connections, 0),
  )

  const hasProjects = computed(() => projects.value.length > 0)

  const randomQuote = computed(() => {
    if (quotes.value.length === 0) return ""
    const idx = Math.floor(Math.random() * quotes.value.length)
    return quotes.value[idx]
  })

  // === Actions ===
  async function load(): Promise<void> {
    status.value = "loading"
    error.value = ""
    try {
      // 模拟网络延迟，便于展示 Loading State
      await new Promise((resolve) => setTimeout(resolve, 600))
      projects.value = structuredClone(mockProjects)
      tunnels.value = structuredClone(mockTunnels)
      servers.value = structuredClone(mockServers)
      activities.value = structuredClone(mockActivities)
      statistics.value = { ...mockStatistics }
      resource.value = { ...mockResource }
      news.value = structuredClone(mockNews)
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

  function togglePin(projectId: string): void {
    const p = projects.value.find((x) => x.id === projectId)
    if (p) p.pinned = !p.pinned
  }

  function toggleFavorite(projectId: string): void {
    const p = projects.value.find((x) => x.id === projectId)
    if (p) p.favorite = !p.favorite
  }

  function startTunnel(tunnelId: string): void {
    const t = tunnels.value.find((x) => x.id === tunnelId)
    if (t) {
      t.status = "starting"
      setTimeout(() => {
        if (t) {
          t.status = "online"
          t.uploadSpeed = Math.random() * 50
          t.downloadSpeed = Math.random() * 100
          t.connections = 1
        }
      }, 1200)
    }
  }

  function stopTunnel(tunnelId: string): void {
    const t = tunnels.value.find((x) => x.id === tunnelId)
    if (t) {
      t.status = "offline"
      t.uploadSpeed = 0
      t.downloadSpeed = 0
      t.connections = 0
    }
  }

  function setTunnelStatus(tunnelId: string, newStatus: TunnelStatus): void {
    const t = tunnels.value.find((x) => x.id === tunnelId)
    if (t) t.status = newStatus
  }

  return {
    // state
    projects,
    tunnels,
    servers,
    activities,
    statistics,
    resource,
    news,
    actions,
    quotes,
    status,
    error,
    lastUpdated,
    // getters
    isLoading,
    isError,
    isReady,
    pinnedProjects,
    favoriteProjects,
    runningTunnels,
    onlineServers,
    connectedServerCount,
    totalConnections,
    hasProjects,
    randomQuote,
    // actions
    load,
    refresh,
    togglePin,
    toggleFavorite,
    startTunnel,
    stopTunnel,
    setTunnelStatus,
  }
})
