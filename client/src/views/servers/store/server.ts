import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { i18n } from '@/i18n'
import { GateAppError } from '@/ipc'
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

function t(key: string, params?: Record<string, unknown>): string {
  return (i18n.global as unknown as { t: (key: string, params?: Record<string, unknown>) => string }).t(
    key,
    params,
  )
}

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
      error.value = e instanceof Error ? e.message : t('server.errors.loadFailed')
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
      throw new GateAppError({
        code: 'SERVER_RELOAD_FAILED',
        messageKey: 'server.errors.savedReloadFailed',
        details: { serverId: id },
        timestamp: Date.now(),
      })
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
      throw new GateAppError({
        code: 'SERVER_CONNECTION_TEST_FAILED',
        messageKey: 'server.errors.connectionTestFailed',
        details: { serverId: id, source: result.error },
        timestamp: Date.now(),
      })
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
  const lastConnectedAt = row.lastConnectedAt
    ? formatRelative(row.lastConnectedAt)
    : t('server.neverConnected')
  const ping = row.lastRttMs ?? 0
  const discovery = row.discovery ?? undefined
  const memoryUsed = discovery?.memory?.usedBytes ?? 0
  const memoryTotal = discovery?.memory?.totalBytes ?? 0
  const diskUsed = discovery?.diskUsage?.usedBytes ?? 0
  const diskTotal = discovery?.diskUsage?.totalBytes ?? 0
  const networkReceived = discovery?.networkUsage?.receivedBytes ?? 0
  const networkTransmitted = discovery?.networkUsage?.transmittedBytes ?? 0

  return {
    id: row.id,
    name: row.name,
    kind: normalizeKind(row.kind),
    region: row.region,
    publicIp: discovery?.publicIp || row.host,
    version: discovery?.gateVersion || (row.sessionId ? t('server.connectedVersion') : t('common.unknown')),
    status,
    connectionMethod: 'tcp' as ConnectionMethod,
    ping,
    tags: row.tags ?? [],
    favorite: false,
    recent: false,
    overview: {
      hostname: discovery?.hostname || row.host,
      os: discovery?.os || t('common.unknown'),
      arch: discovery?.architecture || t('common.unknown'),
      cpu: discovery?.cpu,
      memoryUsedBytes: memoryUsed,
      memoryTotalBytes: memoryTotal,
      privateIp: discovery?.privateIp,
      dockerDetected: discovery?.docker?.detected,
      firewallDetected: discovery?.firewall?.detected,
      diskUsedBytes: diskUsed,
      diskTotalBytes: diskTotal,
      networkReceivedBytes: networkReceived,
      networkTransmittedBytes: networkTransmitted,
      rustVersion: discovery?.runtimeVersion || t('common.unknown'),
      serverVersion: discovery?.gateVersion || (row.sessionId ? t('server.authenticated') : t('common.unknown')),
      installTime: createdAt,
      lastOnline: lastConnectedAt,
      lastHeartbeat: row.lastCheckedAt ? formatRelative(row.lastCheckedAt) : t('server.neverConnected'),
    },
    monitor: monitorFromDiscovery(discovery),
    traffic: trafficFromDiscovery(discovery),
    statistics: {
      uptime: discovery?.uptime ?? 0,
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
  if (['personal', 'cloud', 'nas', 'company', 'docker'].includes(kind)) {
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

function monitorFromDiscovery(discovery: RuntimeServerRecord['discovery']) {
  const monitor = emptyMonitor()
  const memoryUsed = discovery?.memory?.usedBytes ?? 0
  const memoryTotal = discovery?.memory?.totalBytes ?? 0
  const diskUsed = discovery?.diskUsage?.usedBytes ?? 0
  const diskTotal = discovery?.diskUsage?.totalBytes ?? 0
  const networkReceived = discovery?.networkUsage?.receivedBytes ?? 0
  const networkTransmitted = discovery?.networkUsage?.transmittedBytes ?? 0

  monitor.memory = {
    percent: percent(memoryUsed, memoryTotal),
    used: bytesToGb(memoryUsed),
    total: bytesToGb(memoryTotal),
    unit: 'GB',
    history: [],
  }
  monitor.disk = {
    percent: percent(diskUsed, diskTotal),
    used: bytesToGb(diskUsed),
    total: bytesToGb(diskTotal),
    unit: 'GB',
    history: [],
  }
  monitor.network.totalDownload = networkReceived
  monitor.network.totalUpload = networkTransmitted
  monitor.traffic.totalDownload = networkReceived
  monitor.traffic.totalUpload = networkTransmitted
  return monitor
}

function trafficFromDiscovery(discovery: RuntimeServerRecord['discovery']) {
  const traffic = emptyTraffic()
  traffic.totalDownload = discovery?.networkUsage?.receivedBytes ?? 0
  traffic.totalUpload = discovery?.networkUsage?.transmittedBytes ?? 0
  return traffic
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
        label: t('server.health.connection'),
        status: healthy ? 'pass' : warning ? 'warn' : failed ? 'fail' : 'pending',
        message:
          lastError ??
          (healthy ? t('server.health.runtimeAuthenticated') : t('server.health.waitingForHost', { host })),
        latency: ping,
        icon: healthy ? 'check-circle' : failed ? 'alert-circle' : 'clock',
      },
    ],
  } as Server['health']
}

function percent(used: number, total: number): number {
  return total > 0 ? Math.min(100, Math.max(0, (used / total) * 100)) : 0
}

function bytesToGb(bytes: number): number {
  return Math.round((bytes / 1024 ** 3) * 10) / 10
}

function toIso(value: number): string {
  return new Date(value).toISOString()
}

function formatRelative(value: number): string {
  const diffSeconds = Math.max(0, Math.round((Date.now() - value) / 1000))
  if (diffSeconds < 60) return t('server.relative.secondsAgo', { count: diffSeconds })
  const minutes = Math.floor(diffSeconds / 60)
  if (minutes < 60) return t('server.relative.minutesAgo', { count: minutes })
  const hours = Math.floor(minutes / 60)
  if (hours < 24) return t('server.relative.hoursAgo', { count: hours })
  return t('server.relative.daysAgo', { count: Math.floor(hours / 24) })
}

export { SERVER_STATUS_CONFIG, isOnlineStatus, isTransitionStatus }
