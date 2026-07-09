import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { tunnelService } from '@/services/tunnel.service'
import { useServerStore } from '@views/servers'
import type { DashboardTunnel } from '@/monitoring/types'
import type {
  Tunnel,
  TunnelFormData,
  TunnelLoadStatus,
  TunnelProtocol,
  TunnelStatus,
} from '../types'
import { TUNNEL_STATUS_CONFIG, isRunningStatus } from '../utils'

export const defaultTunnelForm: TunnelFormData = {
  name: '',
  protocol: 'tcp',
  localHost: '127.0.0.1',
  localPort: null,
  remotePort: null,
  host: '',
  path: '/',
  projectId: '',
  serverName: '',
  autoStart: false,
  remark: '',
  tags: [],
}

export const useTunnelStore = defineStore('tunnel-module', () => {
  const tunnels = ref<Tunnel[]>([])
  const status = ref<TunnelLoadStatus>('idle')
  const error = ref<string>('')
  const lastUpdated = ref<number>(0)

  const isLoading = computed(() => status.value === 'loading')
  const isError = computed(() => status.value === 'error')
  const isReady = computed(() => status.value === 'success')
  const hasTunnels = computed(() => tunnels.value.length > 0)

  const pinnedTunnels = computed(() => tunnels.value.filter((t) => t.pinned))
  const favoriteTunnels = computed(() => tunnels.value.filter((t) => t.favorite))
  const runningTunnels = computed(() => tunnels.value.filter((t) => isRunningStatus(t.status)))
  const stoppedTunnels = computed(() =>
    tunnels.value.filter((t) => t.status === 'stopped' || t.status === 'offline'),
  )
  const recentTunnels = computed(() =>
    [...tunnels.value]
      .sort((a, b) => new Date(b.updatedAt).getTime() - new Date(a.updatedAt).getTime())
      .slice(0, 10),
  )

  const httpTunnels = computed(() => tunnels.value.filter((t) => t.protocol === 'http'))
  const tcpTunnels = computed(() => tunnels.value.filter((t) => t.protocol === 'tcp'))

  const totalConnections = computed(() =>
    tunnels.value.reduce((sum, t) => sum + t.statistics.connections, 0),
  )
  const totalTraffic = computed(() =>
    tunnels.value.reduce((sum, t) => sum + t.traffic.totalUpload + t.traffic.totalDownload, 0),
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

  async function load(): Promise<void> {
    status.value = 'loading'
    error.value = ''
    try {
      const rows = await tunnelService.list()
      tunnels.value = rows.map(mapRuntimeTunnel)
      status.value = 'success'
      lastUpdated.value = Date.now()
    } catch (e) {
      status.value = 'error'
      error.value = e instanceof Error ? e.message : '隧道加载失败'
    }
  }

  async function refresh(): Promise<void> {
    return load()
  }

  async function createTunnel(form: TunnelFormData): Promise<Tunnel> {
    const serverStore = useServerStore()
    if (serverStore.status === 'idle') {
      await serverStore.load()
    }
    if (!serverStore.onlineServers.length || !form.serverName) {
      throw new Error('请先在服务器页面添加并连接一台服务器，然后再创建隧道。')
    }

    const id = await tunnelService.create({
      localPort: form.localPort ?? 0,
      remotePort: form.remotePort ?? 0,
      protocol: form.protocol,
      localHost: form.localHost || '127.0.0.1',
      host: optionalText(form.host),
      path: optionalText(form.path),
    })

    await tunnelService.edit(id, {
      name: form.name.trim(),
      protocol: form.protocol,
      localHost: form.localHost || '127.0.0.1',
      localPort: form.localPort ?? 0,
      remotePort: form.remotePort ?? 0,
      host: optionalText(form.host),
      path: optionalText(form.path),
    })

    await load()
    const created = getById(id)
    if (!created) {
      throw new Error('隧道已保存，但无法从后端重新加载。')
    }

    if (form.autoStart) {
      await startTunnel(created.id)
      return getById(created.id) ?? created
    }

    return created
  }

  async function updateTunnel(id: string, patch: Partial<TunnelFormData>): Promise<void> {
    await tunnelService.edit(id, {
      name: patch.name,
      protocol: patch.protocol,
      localHost: patch.localHost,
      localPort: patch.localPort ?? undefined,
      remotePort: patch.remotePort ?? undefined,
      host: optionalText(patch.host),
      path: optionalText(patch.path),
    })
    await load()
  }

  async function removeTunnel(id: string): Promise<void> {
    await tunnelService.delete(id)
    await load()
  }

  async function startTunnel(id: string): Promise<void> {
    try {
      await tunnelService.start(id)
    } finally {
      await load()
    }
  }

  async function stopTunnel(id: string): Promise<void> {
    try {
      await tunnelService.stop(id)
    } finally {
      await load()
    }
  }

  async function restartTunnel(id: string): Promise<void> {
    try {
      await tunnelService.restart(id)
    } finally {
      await load()
    }
  }

  function cloneTunnel(_id: string): Tunnel | undefined {
    error.value = '该功能暂未实现'
    return undefined
  }

  function togglePin(_id: string): void {
    error.value = '该功能暂未实现'
  }

  function toggleFavorite(_id: string): void {
    error.value = '该功能暂未实现'
  }

  function tick(): void {
    // Realtime samples are supplied by the Rust runtime dashboard.
  }

  return {
    tunnels,
    status,
    error,
    lastUpdated,
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

function mapRuntimeTunnel(row: DashboardTunnel): Tunnel {
  const nowIso = new Date().toISOString()
  const createdAt = nowIso
  const protocol = normalizeProtocol(row.protocol)
  const status = normalizeStatus(row.status)

  return {
    id: row.id,
    name: row.name,
    protocol,
    localHost: row.localHost ?? '127.0.0.1',
    localPort: row.localPort ?? 0,
    remotePort: row.remotePort ?? 0,
    host: row.host ?? null,
    path: row.path ?? null,
    publicAddr: publicAddress(row),
    remark: '',
    status,
    autoStart: false,
    compression: false,
    encryption: false,
    tags: [],
    serverName: '',
    projectName: '',
    projectId: '',
    pinned: false,
    favorite: false,
    traffic: {
      uploadSpeed: row.uploadSpeedBps,
      downloadSpeed: row.downloadSpeedBps,
      totalUpload: 0,
      totalDownload: 0,
      todayUpload: 0,
      todayDownload: 0,
      history: [],
    },
    statistics: {
      uptime: row.uptimeSeconds,
      connections: row.connections,
      totalConnections: row.connections,
      requests: row.requestCount ?? 0,
      avgLatency: row.averageResponseTimeMs ?? 0,
      peakSpeed: Math.max(row.uploadSpeedBps, row.downloadSpeedBps),
    },
    connections: [],
    logs: (row.recentLogs ?? []).map((log) => ({
      id: `${row.id}-${log.source}-${log.timestamp}-${log.message}`,
      level: normalizeLogLevel(log.level),
      message: log.message,
      timestamp: log.timestamp,
      source: log.source,
    })),
    lastStartedAt: row.uptimeSeconds > 0 ? `${row.uptimeSeconds}s` : '',
    createdAt,
    updatedAt: nowIso,
  }
}

function normalizeProtocol(protocol: DashboardTunnel['protocol']): TunnelProtocol {
  if (protocol === 'http' || protocol === 'tcp' || protocol === 'https' || protocol === 'udp') {
    return protocol
  }
  return 'tcp'
}

function normalizeStatus(status: DashboardTunnel['status']): TunnelStatus {
  if (status === 'running') return 'running'
  if (status === 'warning') return 'error'
  return 'stopped'
}

function normalizeLogLevel(level: string): Tunnel['logs'][number]['level'] {
  if (
    level === 'debug' ||
    level === 'info' ||
    level === 'warn' ||
    level === 'error' ||
    level === 'success'
  ) {
    return level
  }
  return 'info'
}

function publicAddress(row: DashboardTunnel): string {
  if ((row.protocol === 'http' || row.protocol === 'https') && row.host) {
    return `${row.host}${row.path ?? '/'}`
  }

  if (row.remotePort) {
    return `:${row.remotePort}`
  }

  return 'Not assigned'
}

function optionalText(value: string | undefined): string | undefined {
  const trimmed = value?.trim()
  return trimmed ? trimmed : undefined
}

export { TUNNEL_STATUS_CONFIG }
