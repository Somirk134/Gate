import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { serverService, type RuntimeServerRecord } from '@/services/server.service'
import type {
  ConnectionMethod,
  Server,
  ServerFormData,
  ServerKind,
  ServerLoadStatus,
  ServerStatus,
} from '../types'
import { SERVER_STATUS_CONFIG, isOnlineStatus, isTransitionStatus } from '../utils'

export const defaultServerForm: ServerFormData = {
  name: '',
  kind: 'personal',
  host: '',
  port: 7000,
  token: '',
  region: '',
  remark: '',
  tags: [],
  heartbeatInterval: 30,
  reconnectInterval: 5,
  autoConnect: false,
}

export const useServerStore = defineStore('server-module', () => {
  const servers = ref<Server[]>([])
  const status = ref<ServerLoadStatus>('idle')
  const error = ref<string>('')
  const lastUpdated = ref<number>(0)
  const activeServerId = ref<string | null>(null)

  const isLoading = computed(() => status.value === 'loading')
  const isError = computed(() => status.value === 'error')
  const isReady = computed(() => status.value === 'success')
  const hasServers = computed(() => servers.value.length > 0)
  const onlineServers = computed(() => servers.value.filter((s) => isOnlineStatus(s.status)))
  const offlineServers = computed(() => servers.value.filter((s) => !isOnlineStatus(s.status)))
  const errorServers = computed(() => servers.value.filter((s) => s.status === 'error'))
  const favoriteServers = computed(() => servers.value.filter((s) => s.favorite))
  const recentServers = computed(() => servers.value.slice(0, 10))
  const unhealthyServers = computed(() =>
    servers.value.filter((s) => s.health.overall !== 'healthy'),
  )
  const totalTunnels = computed(() =>
    servers.value.reduce((sum, server) => sum + server.statistics.tunnelCount, 0),
  )
  const totalProjects = computed(() =>
    servers.value.reduce((sum, server) => sum + server.statistics.projectCount, 0),
  )
  const totalConnections = computed(() =>
    servers.value.reduce((sum, server) => sum + server.statistics.totalConnections, 0),
  )
  const totalTraffic = computed(() =>
    servers.value.reduce(
      (sum, server) => sum + server.traffic.totalUpload + server.traffic.totalDownload,
      0,
    ),
  )
  const totalUploadSpeed = computed(() =>
    servers.value.reduce((sum, server) => sum + server.traffic.uploadSpeed, 0),
  )
  const totalDownloadSpeed = computed(() =>
    servers.value.reduce((sum, server) => sum + server.traffic.downloadSpeed, 0),
  )

  function getById(id: string): Server | undefined {
    return servers.value.find((s) => s.id === id)
  }

  async function load(): Promise<void> {
    status.value = 'loading'
    error.value = ''
    try {
      const payload = await serverService.list()
      activeServerId.value = payload.activeServerId ?? null
      servers.value = payload.items.map((row) =>
        mapRuntimeServer(row, payload.activeServerId ?? null),
      )
      status.value = 'success'
      lastUpdated.value = Date.now()
    } catch (e) {
      status.value = 'error'
      error.value = e instanceof Error ? e.message : '服务器加载失败'
    }
  }

  async function refresh(): Promise<void> {
    return load()
  }

  async function createServer(form: ServerFormData): Promise<Server> {
    const id = await serverService.create(form)
    await load()
    const created = getById(id)
    if (!created) {
      throw new Error('服务器已保存，但无法从 Runtime Store 重新读取。')
    }
    return created
  }

  async function updateServer(id: string, patch: Partial<ServerFormData>): Promise<void> {
    await serverService.update(id, patch)
    await load()
  }

  async function removeServer(id: string): Promise<void> {
    await serverService.remove(id)
    await load()
  }

  async function connectServer(id: string): Promise<void> {
    markLocalStatus(id, 'connecting')
    try {
      await serverService.connect(id)
      await load()
    } catch (e) {
      await load()
      throw e
    }
  }

  async function disconnectServer(id: string): Promise<void> {
    await serverService.disconnect(id)
    await load()
  }

  async function restartServer(id: string): Promise<void> {
    await disconnectServer(id)
    await connectServer(id)
  }

  async function checkHealth(id: string): Promise<void> {
    const result = await serverService.test(id)
    await load()
    if (!result.ok) {
      throw new Error(result.error || '服务器连接测试失败')
    }
  }

  function toggleFavorite(id: string): void {
    const server = getById(id)
    if (server) server.favorite = !server.favorite
  }

  function tick(): void {
    // Runtime Store supplies server state; no synthetic monitor samples are generated.
  }

  function markLocalStatus(id: string, nextStatus: ServerStatus) {
    const server = getById(id)
    if (server) {
      server.status = nextStatus
      server.health = healthFromStatus(nextStatus, server.settings.host, null, server.ping)
    }
  }

  return {
    servers,
    status,
    error,
    lastUpdated,
    activeServerId,
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

function mapRuntimeServer(row: RuntimeServerRecord, activeId: string | null): Server {
  const status = normalizeStatus(row.status, row.id === activeId)
  const createdAt = toIso(row.createdAt)
  const updatedAt = toIso(row.updatedAt)
  const lastConnectedAt = row.lastConnectedAt ? formatRelative(row.lastConnectedAt) : '从未连接'
  const ping = row.lastRttMs ?? 0

  return {
    id: row.id,
    name: row.name,
    kind: normalizeKind(row.kind),
    region: row.region,
    publicIp: row.host,
    version: row.sessionId ? '已连接' : '未知',
    status,
    connectionMethod: 'tcp' as ConnectionMethod,
    ping,
    tags: row.tags ?? [],
    favorite: false,
    recent: false,
    overview: {
      hostname: row.host,
      os: '未知',
      arch: '未知',
      rustVersion: '未知',
      serverVersion: row.sessionId ? '已认证' : '未知',
      installTime: createdAt,
      lastOnline: lastConnectedAt,
      lastHeartbeat: row.lastCheckedAt ? formatRelative(row.lastCheckedAt) : '从未连接',
    },
    monitor: emptyMonitor(),
    traffic: emptyTraffic(),
    statistics: {
      uptime: 0,
      tunnelCount: 0,
      projectCount: 0,
      totalConnections: status === 'connected' ? 1 : 0,
      requests: 0,
      avgPing: ping,
      peakSpeed: 0,
    },
    connections: [],
    logs: row.lastError
      ? [
          {
            id: `${row.id}-error`,
            level: 'error',
            message: row.lastError,
            timestamp: row.lastCheckedAt ?? row.updatedAt,
            source: 'runtime',
          },
        ]
      : [],
    health: healthFromStatus(status, row.host, row.lastError ?? null, ping),
    settings: {
      name: row.name,
      host: row.host,
      port: row.port,
      token: row.token,
      remark: row.remark,
      heartbeatInterval: row.heartbeatInterval,
      reconnectInterval: row.reconnectInterval,
      autoConnect: row.autoConnect,
    },
    lastConnectedAt,
    createdAt,
    updatedAt,
  }
}

function normalizeStatus(status: string, isActive: boolean): ServerStatus {
  if (isActive && status === 'connected') return 'connected'
  if (status === 'connecting') return 'connecting'
  if (status === 'error') return 'error'
  if (status === 'connected' && !isActive) return 'disconnected'
  if (status === 'disconnected') return 'disconnected'
  return 'offline'
}

function normalizeKind(kind: string): ServerKind {
  if (['personal', 'cloud', 'nas', 'company', 'docker', 'kubernetes'].includes(kind)) {
    return kind as ServerKind
  }
  return 'personal'
}

function emptyMonitor() {
  return {
    cpu: { percent: 0, used: 0, total: 0, unit: 'GB' as const, history: [] },
    memory: { percent: 0, used: 0, total: 0, unit: 'GB' as const, history: [] },
    disk: { percent: 0, used: 0, total: 0, unit: 'GB' as const, history: [] },
    load: { load1: 0, load5: 0, load15: 0, cores: 0 },
    network: { uploadSpeed: 0, downloadSpeed: 0, totalUpload: 0, totalDownload: 0, history: [] },
    traffic: emptyTraffic(),
    connections: { active: 0, total: 0, failed: 0 },
  }
}

function emptyTraffic() {
  return {
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
}

function healthFromStatus(
  status: ServerStatus,
  host: string,
  lastError: string | null,
  ping: number,
) {
  const healthy = status === 'connected'
  const warning = status === 'connecting' || status === 'disconnected'
  const failed = status === 'error'
  return {
    overall: healthy ? 'healthy' : warning ? 'warning' : failed ? 'critical' : 'unknown',
    score: healthy ? 100 : warning ? 60 : failed ? 20 : 0,
    checkedAt: Date.now(),
    items: [
      {
        key: 'connection',
        label: '连接',
        status: healthy ? 'pass' : warning ? 'warn' : failed ? 'fail' : 'pending',
        message: lastError ?? (healthy ? 'Runtime 已认证' : `等待连接到 ${host}`),
        latency: ping,
        icon: healthy ? 'check-circle' : failed ? 'alert-circle' : 'clock',
      },
    ],
  } as Server['health']
}

function toIso(value: number): string {
  return new Date(value).toISOString()
}

function formatRelative(value: number): string {
  const diffSeconds = Math.max(0, Math.round((Date.now() - value) / 1000))
  if (diffSeconds < 60) return `${diffSeconds} 秒前`
  const minutes = Math.floor(diffSeconds / 60)
  if (minutes < 60) return `${minutes} 分钟前`
  const hours = Math.floor(minutes / 60)
  if (hours < 24) return `${hours} 小时前`
  return `${Math.floor(hours / 24)} 天前`
}

export { SERVER_STATUS_CONFIG, isOnlineStatus, isTransitionStatus }
