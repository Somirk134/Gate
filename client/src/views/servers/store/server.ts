/* ==================================================================
   Server Store
   ------------------------------------------------------------------
   集中管理 Server 模块的所有数据与状态。
   当前从 mock 加载，后续替换为真实 Rust Server 时，只需将 load
   动作改为 API 调用并保持返回类型不变即可无缝迁移。

   状态机：idle → loading → success / error
   包含 monitor tick，模拟实时资源 / 流量 / 日志更新。
   ================================================================== */

import { defineStore } from "pinia"
import { ref, computed } from "vue"
import type {
  HealthItemStatus,
  Server,
  ServerFormData,
  ServerLoadStatus,
  ServerLog,
} from "../types"
import { mockServers, defaultServerForm } from "../mock"
import {
  SERVER_STATUS_CONFIG,
  genId,
  isOnlineStatus,
  isTransitionStatus,
  makeLog,
} from "../utils"

export { defaultServerForm }

export const useServerStore = defineStore("server-module", () => {
  // === State ===
  const servers = ref<Server[]>([])
  const status = ref<ServerLoadStatus>("idle")
  const error = ref<string>("")
  const lastUpdated = ref<number>(0)

  // === Getters ===
  const isLoading = computed(() => status.value === "loading")
  const isError = computed(() => status.value === "error")
  const isReady = computed(() => status.value === "success")
  const hasServers = computed(() => servers.value.length > 0)

  const onlineServers = computed(() =>
    servers.value.filter((s) => isOnlineStatus(s.status)),
  )
  const offlineServers = computed(() =>
    servers.value.filter(
      (s) => s.status === "offline" || s.status === "disconnected",
    ),
  )
  const errorServers = computed(() =>
    servers.value.filter((s) => s.status === "error"),
  )
  const favoriteServers = computed(() => servers.value.filter((s) => s.favorite))
  const recentServers = computed(() =>
    [...servers.value]
      .sort((a, b) => new Date(b.updatedAt).getTime() - new Date(a.updatedAt).getTime())
      .slice(0, 10),
  )
  const unhealthyServers = computed(
    () => servers.value.filter((s) => s.health.overall !== "healthy"),
  )

  const totalTunnels = computed(() =>
    servers.value.reduce((sum, s) => sum + s.statistics.tunnelCount, 0),
  )
  const totalProjects = computed(() =>
    servers.value.reduce((sum, s) => sum + s.statistics.projectCount, 0),
  )
  const totalConnections = computed(() =>
    servers.value.reduce((sum, s) => sum + s.monitor.connections.active, 0),
  )
  const totalTraffic = computed(() =>
    servers.value.reduce(
      (sum, s) => sum + s.traffic.totalUpload + s.traffic.totalDownload,
      0,
    ),
  )
  const totalUploadSpeed = computed(() =>
    servers.value.reduce((sum, s) => sum + s.traffic.uploadSpeed, 0),
  )
  const totalDownloadSpeed = computed(() =>
    servers.value.reduce((sum, s) => sum + s.traffic.downloadSpeed, 0),
  )

  function getById(id: string): Server | undefined {
    return servers.value.find((s) => s.id === id)
  }

  // === Actions ===
  async function load(): Promise<void> {
    status.value = "loading"
    error.value = ""
    try {
      await new Promise((resolve) => setTimeout(resolve, 500))
      servers.value = structuredClone(mockServers)
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

  /** 创建服务器 */
  function createServer(form: ServerFormData): Server {
    const nowTs = Date.now()
    const traffic = {
      uploadSpeed: 0,
      downloadSpeed: 0,
      totalUpload: 0,
      totalDownload: 0,
      todayUpload: 0,
      todayDownload: 0,
      weekUpload: 0,
      weekDownload: 0,
      monthUpload: 0,
      monthDownload: 0,
      history: [],
    }
    const server: Server = {
      id: genId(),
      name: form.name.trim(),
      kind: form.kind,
      region: form.region.trim() || "Unknown",
      publicIp: form.host.trim(),
      version: "v0.2.1",
      status: "disconnected",
      connectionMethod: "wss",
      ping: 0,
      tags: [...form.tags],
      favorite: false,
      recent: true,
      overview: {
        hostname: form.name.trim(),
        os: "—",
        arch: "—",
        rustVersion: "1.78.0",
        serverVersion: "v0.2.1",
        installTime: new Date(nowTs).toISOString(),
        lastOnline: "—",
        lastHeartbeat: "—",
      },
      monitor: {
        cpu: { percent: 0, used: 0, total: 0, unit: "GB", history: [] },
        memory: { percent: 0, used: 0, total: 0, unit: "GB", history: [] },
        disk: { percent: 0, used: 0, total: 0, unit: "GB", history: [] },
        load: { load1: 0, load5: 0, load15: 0, cores: 0 },
        network: {
          uploadSpeed: 0,
          downloadSpeed: 0,
          totalUpload: 0,
          totalDownload: 0,
          history: [],
        },
        traffic,
        connections: { active: 0, total: 0, failed: 0 },
      },
      traffic,
      statistics: {
        uptime: 0,
        tunnelCount: 0,
        projectCount: 0,
        totalConnections: 0,
        requests: 0,
        avgPing: 0,
        peakSpeed: 0,
      },
      connections: [],
      logs: [
        makeLog("info", "server registered, waiting to connect", "gateway"),
      ],
      health: {
        overall: "unknown",
        score: 0,
        checkedAt: nowTs,
        items: [],
      },
      settings: {
        name: form.name.trim(),
        host: form.host.trim(),
        port: form.port ?? 7000,
        token: form.token.trim(),
        remark: form.remark.trim(),
        heartbeatInterval: form.heartbeatInterval,
        reconnectInterval: form.reconnectInterval,
        autoConnect: form.autoConnect,
      },
      lastConnectedAt: "—",
      createdAt: new Date(nowTs).toISOString(),
      updatedAt: new Date(nowTs).toISOString(),
    }
    servers.value.unshift(server)
    return server
  }

  /** 更新服务器 */
  function updateServer(id: string, patch: Partial<ServerFormData>): void {
    const s = servers.value.find((x) => x.id === id)
    if (!s) return
    if (patch.name !== undefined) {
      s.name = patch.name.trim()
      s.settings.name = patch.name.trim()
    }
    if (patch.kind !== undefined) s.kind = patch.kind
    if (patch.host !== undefined) {
      s.settings.host = patch.host
      s.publicIp = patch.host
    }
    if (patch.port !== undefined) s.settings.port = patch.port ?? 7000
    if (patch.token !== undefined) s.settings.token = patch.token
    if (patch.region !== undefined) s.region = patch.region
    if (patch.remark !== undefined) s.settings.remark = patch.remark.trim()
    if (patch.tags !== undefined) s.tags = [...patch.tags]
    if (patch.heartbeatInterval !== undefined)
      s.settings.heartbeatInterval = patch.heartbeatInterval
    if (patch.reconnectInterval !== undefined)
      s.settings.reconnectInterval = patch.reconnectInterval
    if (patch.autoConnect !== undefined) s.settings.autoConnect = patch.autoConnect
    s.updatedAt = new Date().toISOString()
  }

  /** 删除服务器 */
  function removeServer(id: string): void {
    const idx = servers.value.findIndex((x) => x.id === id)
    if (idx !== -1) servers.value.splice(idx, 1)
  }

  /** 连接服务器 */
  function connectServer(id: string): void {
    const s = servers.value.find((x) => x.id === id)
    if (!s) return
    s.status = "connecting"
    pushLog(s, "info", `connecting to ${s.name}…`, "gateway")
    setTimeout(() => {
      if (!s) return
      s.status = "connected"
      s.ping = Math.floor(Math.random() * 200) + 20
      s.monitor.cpu.percent = Math.floor(Math.random() * 40) + 10
      s.monitor.memory.percent = Math.floor(Math.random() * 30) + 30
      s.monitor.network.uploadSpeed = Math.floor(Math.random() * 200 * 1024)
      s.monitor.network.downloadSpeed = Math.floor(Math.random() * 500 * 1024)
      s.traffic.uploadSpeed = s.monitor.network.uploadSpeed
      s.traffic.downloadSpeed = s.monitor.network.downloadSpeed
      s.lastConnectedAt = "刚刚"
      s.overview.lastOnline = "刚刚"
      s.overview.lastHeartbeat = "刚刚"
      pushLog(s, "success", "connection established, server online", "transport")
    }, 900)
  }

  /** 断开服务器 */
  function disconnectServer(id: string): void {
    const s = servers.value.find((x) => x.id === id)
    if (!s) return
    s.status = "disconnected"
    s.ping = 0
    s.traffic.uploadSpeed = 0
    s.traffic.downloadSpeed = 0
    s.monitor.network.uploadSpeed = 0
    s.monitor.network.downloadSpeed = 0
    s.monitor.connections.active = 0
    s.connections = []
    pushLog(s, "info", "disconnected from server", "gateway")
  }

  /** 重启服务器 */
  function restartServer(id: string): void {
    const s = servers.value.find((x) => x.id === id)
    if (!s) return
    s.status = "connecting"
    pushLog(s, "info", "restarting server…", "system")
    setTimeout(() => {
      if (!s) return
      s.status = "connected"
      s.lastConnectedAt = "刚刚"
      pushLog(s, "success", "server restarted successfully", "system")
    }, 1200)
  }

  /** 执行健康检查（Mock） */
  function checkHealth(id: string): void {
    const s = servers.value.find((x) => x.id === id)
    if (!s) return
    // 标记为 pending
    s.health.items = s.health.items.map((item) => ({
      ...item,
      status: "pending" as const,
    }))
    pushLog(s, "info", "running health check…", "system")
    setTimeout(() => {
      if (!s) return
      const online = isOnlineStatus(s.status)
      const passCount = online ? s.health.items.length : s.health.items.length - 3
      const score = Math.round((passCount / s.health.items.length) * 100)
      s.health.overall = !online ? "critical" : score >= 90 ? "healthy" : "warning"
      s.health.score = score
      s.health.checkedAt = Date.now()
      s.health.items = s.health.items.map((item) => ({
        ...item,
        status: (
          item.key === "online" || item.key === "api" || item.key === "tunnel"
            ? online
              ? "pass"
              : "fail"
            : "pass"
        ) as HealthItemStatus,
      }))
      pushLog(s, "success", `health check complete: ${score}/100`, "system")
    }, 1000)
  }

  /** 切换收藏 */
  function toggleFavorite(id: string): void {
    const s = servers.value.find((x) => x.id === id)
    if (s) s.favorite = !s.favorite
  }

  /** 推送日志（内部） */
  function pushLog(
    s: Server,
    level: ServerLog["level"],
    message: string,
    source: string,
  ): void {
    s.logs.push({
      id: genId("slog"),
      level,
      message,
      timestamp: Date.now(),
      source,
    })
    if (s.logs.length > 200) s.logs.splice(0, s.logs.length - 200)
  }

  /**
   * Monitor tick —— 模拟实时数据更新。
   * 由 useServerMonitor 在组件层定时调用。
   * 仅对已连接服务器抖动资源 / 流量 / 时长 / 连接。
   */
  function tick(): void {
    for (const s of servers.value) {
      if (!isOnlineStatus(s.status)) continue

      // CPU 抖动
      const cpuJitter = (Math.random() - 0.45) * 12
      s.monitor.cpu.percent = Math.max(
        2,
        Math.min(98, Math.round(s.monitor.cpu.percent + cpuJitter)),
      )
      s.monitor.cpu.history.push(s.monitor.cpu.percent)
      if (s.monitor.cpu.history.length > 12) s.monitor.cpu.history.shift()

      // Memory 抖动
      const memJitter = (Math.random() - 0.48) * 6
      s.monitor.memory.percent = Math.max(
        10,
        Math.min(95, Math.round(s.monitor.memory.percent + memJitter)),
      )
      s.monitor.memory.history.push(s.monitor.memory.percent)
      if (s.monitor.memory.history.length > 12) s.monitor.memory.history.shift()

      // 速度抖动
      const upJitter = (Math.random() - 0.4) * 60 * 1024
      const downJitter = (Math.random() - 0.4) * 150 * 1024
      s.traffic.uploadSpeed = Math.max(0, Math.floor(s.traffic.uploadSpeed + upJitter))
      s.traffic.downloadSpeed = Math.max(
        0,
        Math.floor(s.traffic.downloadSpeed + downJitter),
      )
      s.monitor.network.uploadSpeed = s.traffic.uploadSpeed
      s.monitor.network.downloadSpeed = s.traffic.downloadSpeed

      // 累计流量
      s.traffic.totalUpload += s.traffic.uploadSpeed
      s.traffic.totalDownload += s.traffic.downloadSpeed
      s.traffic.todayUpload += s.traffic.uploadSpeed
      s.traffic.todayDownload += s.traffic.downloadSpeed
      s.monitor.network.totalUpload += s.traffic.uploadSpeed
      s.monitor.network.totalDownload += s.traffic.downloadSpeed

      // 峰值
      if (s.traffic.downloadSpeed > s.statistics.peakSpeed) {
        s.statistics.peakSpeed = s.traffic.downloadSpeed
      }

      // 时长 +1
      s.statistics.uptime += 1

      // 推入流量采样（保留 12 个）
      const d = new Date()
      const label = `${String(d.getHours()).padStart(2, "0")}:${String(d.getMinutes()).padStart(2, "0")}`
      s.traffic.history.push({
        time: label,
        upload: s.traffic.uploadSpeed,
        download: s.traffic.downloadSpeed,
      })
      if (s.traffic.history.length > 12) s.traffic.history.shift()

      // 连接时长 +1
      for (const c of s.connections) {
        c.duration += 1
      }

      // 偶发推送日志
      if (Math.random() < 0.3) {
        const samples: Array<{ level: ServerLog["level"]; message: string; source: string }> = [
          { level: "debug", message: "heartbeat packet sent", source: "transport" },
          { level: "info", message: "new connection from client", source: "gateway" },
          { level: "info", message: "connection closed by peer", source: "gateway" },
          { level: "debug", message: "request forwarded to tunnel", source: "gateway" },
          { level: "success", message: "health check passed", source: "system" },
        ]
        const sample = samples[Math.floor(Math.random() * samples.length)]
        pushLog(s, sample.level, sample.message, sample.source)
      }
    }
  }

  return {
    // state
    servers,
    status,
    error,
    lastUpdated,
    // getters
    isLoading,
    isError,
    isReady,
    hasServers,
    onlineServers,
    offlineServers,
    errorServers,
    favoriteServers,
    recentServers,
    unhealthyServers,
    totalTunnels,
    totalProjects,
    totalConnections,
    totalTraffic,
    totalUploadSpeed,
    totalDownloadSpeed,
    getById,
    // actions
    load,
    refresh,
    createServer,
    updateServer,
    removeServer,
    connectServer,
    disconnectServer,
    restartServer,
    checkHealth,
    toggleFavorite,
    tick,
  }
})

/* 导出状态配置，便于组件直接引用 */
export { SERVER_STATUS_CONFIG }

/* 重新导出工具，供组件复用 */
export { isOnlineStatus, isTransitionStatus }
