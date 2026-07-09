import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { dashboardService } from '@/monitoring/services'
import type { DashboardData, DashboardTunnel as RuntimeTunnel } from '@/monitoring/types'
import type {
  DashboardActivity,
  DashboardLoadStatus,
  DashboardNews,
  DashboardProject,
  DashboardQuickAction,
  DashboardResource,
  DashboardServer,
  DashboardStatistics,
  DashboardTunnel,
  CertificateStatus,
  TunnelStatus,
} from '../types'

export const useDashboardStore = defineStore('dashboard', () => {
  const projects = ref<DashboardProject[]>([])
  const tunnels = ref<DashboardTunnel[]>([])
  const servers = ref<DashboardServer[]>([])
  const activities = ref<DashboardActivity[]>([])
  const statistics = ref<DashboardStatistics | null>(null)
  const resource = ref<DashboardResource | null>(null)
  const news = ref<DashboardNews[]>([])
  const actions = ref<DashboardQuickAction[]>([])
  const quotes = ref<string[]>([])

  const status = ref<DashboardLoadStatus>('idle')
  const error = ref<string>('')
  const lastUpdated = ref<number>(0)

  const isLoading = computed(() => status.value === 'loading')
  const isError = computed(() => status.value === 'error')
  const isReady = computed(() => status.value === 'success')
  const pinnedProjects = computed(() => projects.value.filter((p) => p.pinned))
  const favoriteProjects = computed(() => projects.value.filter((p) => p.favorite))
  const runningTunnels = computed(() =>
    tunnels.value.filter((t) => t.status === 'online' || t.status === 'connecting'),
  )
  const onlineServers = computed(() => servers.value.filter((s) => s.status === 'online'))
  const connectedServerCount = computed(() => servers.value.filter((s) => s.connected).length)
  const totalConnections = computed(() => tunnels.value.reduce((sum, t) => sum + t.connections, 0))
  const hasProjects = computed(() => projects.value.length > 0)
  const randomQuote = computed(() => quotes.value[0] ?? '')

  async function load(): Promise<void> {
    status.value = 'loading'
    error.value = ''
    try {
      const data = await dashboardService.getDashboard()
      applyDashboard(data)
      status.value = 'success'
      lastUpdated.value = Date.now()
    } catch (e) {
      status.value = 'error'
      error.value = e instanceof Error ? e.message : '首页数据加载失败'
    }
  }

  async function refresh(): Promise<void> {
    return load()
  }

  function togglePin(_projectId: string): void {
    error.value = '该功能暂未实现'
  }

  function toggleFavorite(_projectId: string): void {
    error.value = '该功能暂未实现'
  }

  async function startTunnel(_tunnelId: string): Promise<void> {
    error.value = '请在隧道页面执行启动操作'
  }

  async function stopTunnel(_tunnelId: string): Promise<void> {
    error.value = '请在隧道页面执行停止操作'
  }

  function setTunnelStatus(_tunnelId: string, _newStatus: TunnelStatus): void {
    error.value = '该功能暂未实现'
  }

  function applyDashboard(data: DashboardData) {
    projects.value = []
    servers.value = []
    news.value = []
    actions.value = []
    quotes.value = []
    tunnels.value = data.tunnels.map(mapTunnel)
    activities.value = data.recentActivity.map((activity) => ({
      id: activity.id,
      type: 'update',
      title: activity.title,
      description: activity.category,
      timestamp: activity.timestamp,
    }))
    statistics.value = {
      projectCount: 0,
      tunnelCount: data.overview.tunnelCount,
      runningTunnel: data.overview.runningTunnel,
      todayUpload: data.statistics.traffic.todayTrafficBytes,
      todayDownload: 0,
      onlineTime: data.statistics.client.onlineTimeSeconds,
    }
    resource.value = {
      cpu: data.statistics.system.cpuUsage,
      memory: data.statistics.system.memoryUsage,
      traffic: data.statistics.traffic.totalTrafficBytes,
      connection: data.statistics.connection.currentConnection,
    }
  }

  return {
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
    load,
    refresh,
    togglePin,
    toggleFavorite,
    startTunnel,
    stopTunnel,
    setTunnelStatus,
  }
})

function mapTunnel(tunnel: RuntimeTunnel): DashboardTunnel {
  const https =
    tunnel.protocol === 'https'
      ? {
          certificateStatus: mapCertificateStatus(tunnel.tls?.certificateStatus),
          expireDays: tunnel.tls?.certificateExpireDays ?? 0,
          issuer: tunnel.tls?.certificateIssuer ?? '',
          tlsVersion: tunnel.tls?.tlsVersion ?? 'TLS',
          handshakeCount: tunnel.tls?.handshakeCount ?? 0,
          httpsTraffic: tunnel.trafficBytes ?? 0,
          errorCount: tunnel.tls?.errorCount ?? 0,
        }
      : undefined

  return {
    id: tunnel.id,
    name: tunnel.name,
    protocol: tunnel.protocol,
    status: tunnel.status === 'running' ? 'online' : 'offline',
    localPort: tunnel.localPort ?? 0,
    publicPort: tunnel.remotePort ?? 0,
    publicHost: tunnel.host ?? '',
    uploadSpeed: tunnel.uploadSpeedBps / 1024,
    downloadSpeed: tunnel.downloadSpeedBps / 1024,
    connections: tunnel.connections,
    https,
  }
}

function mapCertificateStatus(status: string | undefined): CertificateStatus {
  switch (status) {
    case 'active':
      return 'active'
    case 'expiringSoon':
    case 'expiring_soon':
      return 'expiring_soon'
    case 'expired':
      return 'expired'
    case 'failed':
    case 'error':
      return 'error'
    default:
      return 'missing'
  }
}
