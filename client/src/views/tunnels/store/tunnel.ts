/* ==================================================================
   Tunnel Store
   ------------------------------------------------------------------
   集中管理 Tunnel 模块的所有数据与状态。
   当前从 mock 加载，后续替换为真实 Tunnel Engine 时，只需将 load
   动作改为 API 调用并保持返回类型不变即可无缝迁移。

   状态机：idle → loading → success / error
   包含 monitor tick，模拟实时流量 / 连接 / 日志更新。
   ================================================================== */

import { defineStore } from "pinia"
import { ref, computed } from "vue"
import { tunnelService } from "@/services/tunnel.service"
import type {
  Tunnel,
  TunnelFormData,
  TunnelLoadStatus,
  TunnelLog,
} from "../types"
import {
  mockTunnels,
  mockProjects,
  mockServerNames,
  defaultTunnelForm,
} from "../mock"
import {
  TUNNEL_STATUS_CONFIG,
  genId,
  isRunningStatus,
} from "../utils"

/* 复用项目 / 服务器名（供创建对话框选项） */
export { mockProjects, mockServerNames, defaultTunnelForm }

export const useTunnelStore = defineStore("tunnel-module", () => {
  // === State ===
  const tunnels = ref<Tunnel[]>([])
  const status = ref<TunnelLoadStatus>("idle")
  const error = ref<string>("")
  const lastUpdated = ref<number>(0)

  // === Getters ===
  const isLoading = computed(() => status.value === "loading")
  const isError = computed(() => status.value === "error")
  const isReady = computed(() => status.value === "success")
  const hasTunnels = computed(() => tunnels.value.length > 0)

  const pinnedTunnels = computed(() => tunnels.value.filter((t) => t.pinned))
  const favoriteTunnels = computed(() => tunnels.value.filter((t) => t.favorite))
  const runningTunnels = computed(() =>
    tunnels.value.filter((t) => isRunningStatus(t.status)),
  )
  const stoppedTunnels = computed(() =>
    tunnels.value.filter((t) => t.status === "stopped" || t.status === "offline"),
  )
  const recentTunnels = computed(() =>
    [...tunnels.value]
      .sort((a, b) => new Date(b.updatedAt).getTime() - new Date(a.updatedAt).getTime())
      .slice(0, 10),
  )

  const httpTunnels = computed(() => tunnels.value.filter((t) => t.protocol === "http"))
  const tcpTunnels = computed(() => tunnels.value.filter((t) => t.protocol === "tcp"))

  const totalConnections = computed(() =>
    tunnels.value.reduce((sum, t) => sum + t.statistics.connections, 0),
  )
  const totalTraffic = computed(() =>
    tunnels.value.reduce(
      (sum, t) => sum + t.traffic.totalUpload + t.traffic.totalDownload,
      0,
    ),
  )
  const totalUploadSpeed = computed(() =>
    tunnels.value.reduce((sum, t) => sum + t.traffic.uploadSpeed, 0),
  )
  const totalDownloadSpeed = computed(() =>
    tunnels.value.reduce((sum, t) => sum + t.traffic.downloadSpeed, 0),
  )

  function getById(id: string): Tunnel | undefined {
    return tunnels.value.find((t) => t.id === id)
  }

  // === Actions ===
  async function load(): Promise<void> {
    status.value = "loading"
    error.value = ""
    try {
      await new Promise((resolve) => setTimeout(resolve, 500))
      tunnels.value = structuredClone(mockTunnels)
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

  /** 创建隧道 */
  async function createTunnel(form: TunnelFormData): Promise<Tunnel> {
    const nowTs = Date.now()
    const project = mockProjects.find((p) => p.id === form.projectId)
    const id = await tunnelService.create({
      localPort: form.localPort ?? 0,
      remotePort: form.remotePort ?? 0,
      protocol: form.protocol,
      localHost: form.localHost || "127.0.0.1",
      host: form.protocol === "http" ? "example.com" : undefined,
      path: form.protocol === "http" ? "/" : undefined,
    })
    const tunnel: Tunnel = {
      id,
      name: form.name.trim(),
      protocol: form.protocol,
      localHost: form.localHost || "127.0.0.1",
      localPort: form.localPort ?? 0,
      remotePort: form.remotePort ?? 0,
      publicAddr: `gate.dev:${form.remotePort ?? 0}`,
      remark: form.remark.trim(),
      status: "stopped",
      autoStart: form.autoStart,
      compression: false,
      encryption: false,
      tags: [...form.tags],
      serverName: form.serverName,
      projectName: project?.name ?? "—",
      projectId: form.projectId,
      pinned: false,
      favorite: false,
      traffic: {
        uploadSpeed: 0,
        downloadSpeed: 0,
        totalUpload: 0,
        totalDownload: 0,
        todayUpload: 0,
        todayDownload: 0,
        history: [],
      },
      statistics: {
        uptime: 0,
        connections: 0,
        totalConnections: 0,
        requests: 0,
        avgLatency: 0,
        peakSpeed: 0,
      },
      connections: [],
      logs: [
        {
          id: genId("log"),
          level: "info",
          message: "tunnel created, waiting to start",
          timestamp: nowTs,
          source: "frpc",
        },
      ],
      lastStartedAt: "—",
      createdAt: new Date(nowTs).toISOString(),
      updatedAt: new Date(nowTs).toISOString(),
    }
    tunnels.value.unshift(tunnel)
    if (form.autoStart) {
      void startTunnel(tunnel.id)
    }
    return tunnel
  }

  /** 更新隧道 */
  function updateTunnel(id: string, patch: Partial<TunnelFormData>): void {
    const t = tunnels.value.find((x) => x.id === id)
    if (!t) return
    if (patch.name !== undefined) t.name = patch.name.trim()
    if (patch.protocol !== undefined) t.protocol = patch.protocol
    if (patch.localHost !== undefined) t.localHost = patch.localHost
    if (patch.localPort !== undefined) t.localPort = patch.localPort ?? 0
    if (patch.remotePort !== undefined) {
      t.remotePort = patch.remotePort ?? 0
      t.publicAddr = `gate.dev:${t.remotePort}`
    }
    if (patch.projectId !== undefined) {
      t.projectId = patch.projectId
      const p = mockProjects.find((x) => x.id === patch.projectId)
      t.projectName = p?.name ?? "—"
    }
    if (patch.serverName !== undefined) t.serverName = patch.serverName
    if (patch.autoStart !== undefined) t.autoStart = patch.autoStart
    if (patch.remark !== undefined) t.remark = patch.remark.trim()
    if (patch.tags !== undefined) t.tags = [...patch.tags]
    t.updatedAt = new Date().toISOString()
  }

  /** 删除隧道 */
  async function removeTunnel(id: string): Promise<void> {
    await tunnelService.delete(id)
    const idx = tunnels.value.findIndex((x) => x.id === id)
    if (idx !== -1) tunnels.value.splice(idx, 1)
  }

  /** 启动隧道 */
  async function startTunnel(id: string): Promise<void> {
    const t = tunnels.value.find((x) => x.id === id)
    if (!t) return
    await tunnelService.start(id)
    t.status = "starting"
    pushLog(t, "info", `starting tunnel ${t.name}…`, "frpc")
    setTimeout(() => {
      if (!t) return
      t.status = "running"
      t.lastStartedAt = "刚刚"
      t.statistics.uptime = 1
      t.traffic.uploadSpeed = Math.floor(Math.random() * 50 * 1024)
      t.traffic.downloadSpeed = Math.floor(Math.random() * 120 * 1024)
      pushLog(t, "success", "tunnel established, public address ready", "transport")
    }, 900)
  }

  /** 停止隧道 */
  async function stopTunnel(id: string): Promise<void> {
    const t = tunnels.value.find((x) => x.id === id)
    if (!t) return
    await tunnelService.stop(id)
    t.status = "stopping"
    pushLog(t, "info", "stopping tunnel…", "frpc")
    setTimeout(() => {
      if (!t) return
      t.status = "stopped"
      t.lastStartedAt = "—"
      t.traffic.uploadSpeed = 0
      t.traffic.downloadSpeed = 0
      t.statistics.connections = 0
      t.connections = []
      pushLog(t, "info", "tunnel stopped", "frpc")
    }, 600)
  }

  /** 重启隧道 */
  async function restartTunnel(id: string): Promise<void> {
    const t = tunnels.value.find((x) => x.id === id)
    if (!t) return
    await tunnelService.restart(id)
    t.status = "restarting"
    pushLog(t, "info", "restarting tunnel…", "frpc")
    setTimeout(() => {
      if (!t) return
      t.status = "running"
      t.lastStartedAt = "刚刚"
      t.statistics.uptime = 1
      pushLog(t, "success", "tunnel restarted successfully", "transport")
    }, 1000)
  }

  /** 克隆隧道 */
  function cloneTunnel(id: string): Tunnel | undefined {
    const src = tunnels.value.find((x) => x.id === id)
    if (!src) return undefined
    const nowTs = Date.now()
    const clone: Tunnel = {
      ...structuredClone(src),
      id: genId(),
      name: `${src.name}-copy`,
      pinned: false,
      favorite: false,
      status: "stopped",
      lastStartedAt: "—",
      createdAt: new Date(nowTs).toISOString(),
      updatedAt: new Date(nowTs).toISOString(),
      traffic: {
        uploadSpeed: 0,
        downloadSpeed: 0,
        totalUpload: 0,
        totalDownload: 0,
        todayUpload: 0,
        todayDownload: 0,
        history: [],
      },
      statistics: {
        uptime: 0,
        connections: 0,
        totalConnections: 0,
        requests: 0,
        avgLatency: 0,
        peakSpeed: 0,
      },
      connections: [],
      logs: [
        {
          id: genId("log"),
          level: "info",
          message: `cloned from ${src.name}`,
          timestamp: nowTs,
          source: "frpc",
        },
      ],
    }
    const idx = tunnels.value.findIndex((x) => x.id === id)
    tunnels.value.splice(idx + 1, 0, clone)
    return clone
  }

  /** 切换固定 */
  function togglePin(id: string): void {
    const t = tunnels.value.find((x) => x.id === id)
    if (t) t.pinned = !t.pinned
  }

  /** 切换收藏 */
  function toggleFavorite(id: string): void {
    const t = tunnels.value.find((x) => x.id === id)
    if (t) t.favorite = !t.favorite
  }

  /** 推送日志（内部） */
  function pushLog(
    t: Tunnel,
    level: TunnelLog["level"],
    message: string,
    source: string,
  ): void {
    t.logs.push({
      id: genId("log"),
      level,
      message,
      timestamp: Date.now(),
      source,
    })
    // 限制日志条数，避免无限增长
    if (t.logs.length > 200) t.logs.splice(0, t.logs.length - 200)
  }

  /**
   * Monitor tick —— 模拟实时数据更新。
   * 由 useTunnelMonitor 在组件层定时调用。
   * 仅对运行中隧道抖动流量 / 累计 / 时长 / 连接。
   */
  function tick(): void {
    for (const t of tunnels.value) {
      if (!isRunningStatus(t.status)) continue
      if (t.status !== "running") continue

      // 速度抖动
      const upJitter = (Math.random() - 0.4) * 30 * 1024
      const downJitter = (Math.random() - 0.4) * 80 * 1024
      t.traffic.uploadSpeed = Math.max(0, Math.floor(t.traffic.uploadSpeed + upJitter))
      t.traffic.downloadSpeed = Math.max(0, Math.floor(t.traffic.downloadSpeed + downJitter))

      // 累计流量
      t.traffic.totalUpload += t.traffic.uploadSpeed
      t.traffic.totalDownload += t.traffic.downloadSpeed
      t.traffic.todayUpload += t.traffic.uploadSpeed
      t.traffic.todayDownload += t.traffic.downloadSpeed

      // 峰值
      if (t.traffic.downloadSpeed > t.statistics.peakSpeed) {
        t.statistics.peakSpeed = t.traffic.downloadSpeed
      }

      // 时长 +1
      t.statistics.uptime += 1

      // 推入历史采样点（保留 12 个）
      const d = new Date()
      const label = `${String(d.getHours()).padStart(2, "0")}:${String(d.getMinutes()).padStart(2, "0")}`
      t.traffic.history.push({
        time: label,
        upload: t.traffic.uploadSpeed,
        download: t.traffic.downloadSpeed,
      })
      if (t.traffic.history.length > 12) t.traffic.history.shift()

      // 连接时长 +1
      for (const c of t.connections) {
        c.duration += 1
      }

      // 偶发推送日志
      if (Math.random() < 0.25) {
        const samples: Array<{ level: TunnelLog["level"]; message: string; source: string }> = [
          { level: "debug", message: "heartbeat packet sent", source: "transport" },
          { level: "info", message: "new connection from client", source: "api" },
          { level: "info", message: "connection closed by peer", source: "api" },
          { level: "debug", message: "request forwarded to local service", source: "api" },
        ]
        const s = samples[Math.floor(Math.random() * samples.length)]
        pushLog(t, s.level, s.message, s.source)
      }
    }
  }

  return {
    // state
    tunnels,
    status,
    error,
    lastUpdated,
    // getters
    isLoading,
    isError,
    isReady,
    hasTunnels,
    pinnedTunnels,
    favoriteTunnels,
    runningTunnels,
    stoppedTunnels,
    recentTunnels,
    httpTunnels,
    tcpTunnels,
    totalConnections,
    totalTraffic,
    totalUploadSpeed,
    totalDownloadSpeed,
    getById,
    // actions
    load,
    refresh,
    createTunnel,
    updateTunnel,
    removeTunnel,
    startTunnel,
    stopTunnel,
    restartTunnel,
    cloneTunnel,
    togglePin,
    toggleFavorite,
    tick,
  }
})

/* 导出状态配置，便于组件直接引用 */
export { TUNNEL_STATUS_CONFIG }
