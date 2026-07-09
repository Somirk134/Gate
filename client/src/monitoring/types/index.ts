/** Metric instrument kind supported by Gate Monitoring Center. */
export type MetricKind =
  'counter' | 'gauge' | 'histogram' | 'summary' | 'rate' | 'average' | 'peak' | 'min' | 'max'

/** Logical metric scope. */
export type MetricScope =
  | 'tunnel'
  | 'project'
  | 'server'
  | 'connection'
  | 'heartbeat'
  | 'authentication'
  | 'runtime'
  | 'system'
  | 'network'
  | 'client'

/** Health status emitted by the HealthCenter. */
export type HealthStatus = 'healthy' | 'warning' | 'critical' | 'offline'

/** Metric label map. */
export interface MetricLabel {
  key: string
  value: string
}

/** Unified metric sample. */
export interface Metric {
  name: string
  description: string
  kind: MetricKind
  scope: MetricScope
  unit: string
  value: number | number[]
  labels: MetricLabel[]
  timestamp: number
}

/** Traffic statistics shared by charts and cards. */
export interface TrafficStatistics {
  uploadBytes: number
  downloadBytes: number
  uploadSpeedBps: number
  downloadSpeedBps: number
  peakSpeedBps: number
  averageSpeedBps: number
  todayTrafficBytes: number
  totalTrafficBytes: number
}

/** Tunnel statistics. */
export interface TunnelStatistics {
  tunnelCount: number
  runningTunnel: number
  stoppedTunnel: number
  upload: number
  download: number
  peakSpeedBps: number
  averageSpeedBps: number
  runningTimeSeconds: number
  todayTraffic: number
  totalTraffic: number
}

/** Connection statistics. */
export interface ConnectionStatistics {
  currentConnection: number
  totalConnection: number
  success: number
  failure: number
  reconnect: number
  disconnect: number
  connectionDurationMs: number
  averageRttMs: number
}

/** Runtime statistics. */
export interface RuntimeStatistics {
  runningTask: number
  workerCount: number
  schedulerQueue: number
  bufferUsage: number
  sessionCount: number
  runtimeUptimeSeconds: number
}

/** TLS statistics sourced from the runtime. */
export interface TlsStatistics {
  sessionCount: number
  handshakeCount: number
  errorCount: number
  trafficBytes: number
}

/** HTTP tunnel statistics sourced from real request logs. */
export interface HttpStatistics {
  requestsTotal: number
  activeRequests: number
  statusCodes: Record<string, number>
  latency: {
    totalMs: number
    averageMs: number
  }
  bandwidth: {
    bytes: number
  }
}

/** System statistics. */
export interface SystemStatistics {
  cpuUsage: number
  memoryUsage: number
  diskUsage?: number
  threadCount: number
  processUptimeSeconds: number
  openFile?: number
}

/** Client statistics. */
export interface ClientStatistics {
  onlineTimeSeconds: number
  openProject: number
  currentWorkspace: string
  uiFps?: number
  memoryBytes: number
}

/** Unified statistics snapshot. */
export interface Statistics {
  collectedAt: number
  tunnel: TunnelStatistics
  traffic: TrafficStatistics
  connection: ConnectionStatistics
  runtime: RuntimeStatistics
  http?: HttpStatistics
  tls?: TlsStatistics
  system: SystemStatistics
  client: ClientStatistics
}

/** Health signal from one subsystem. */
export interface HealthSignal {
  target: 'tunnel' | 'connection' | 'runtime' | 'heartbeat' | 'server' | 'system'
  status: HealthStatus
  message: string
  score: number
  timestamp: number
}

/** Aggregated health report. */
export interface HealthReport {
  overall: HealthStatus
  signals: HealthSignal[]
  updatedAt: number
}

/** Realtime speed point. */
export interface RealtimeSpeedPoint {
  timestamp: number
  uploadBps: number
  downloadBps: number
}

/** Connection trend point. */
export interface ConnectionTrendPoint {
  timestamp: number
  current: number
  success: number
  failure: number
  reconnect: number
}

/** Traffic trend point. */
export interface TrafficTrendPoint {
  timestamp: number
  uploadBytes: number
  downloadBytes: number
}

/** Dashboard tunnel row. */
export interface HttpRequestRecord {
  method: string
  url: string
  host: string
  status: number
  latencyMs: number
  clientIp: string
  trafficBytes: number
  timestamp: number
}

export interface DashboardTunnel {
  id: string
  name: string
  protocol: 'tcp' | 'udp' | 'http' | 'https'
  status: 'running' | 'stopped' | 'warning'
  localHost?: string
  localPort?: number
  remotePort?: number
  host?: string | null
  path?: string | null
  uploadSpeedBps: number
  downloadSpeedBps: number
  connections: number
  uptimeSeconds: number
  requestCount?: number
  successRate?: number
  averageResponseTimeMs?: number
  trafficBytes?: number
  tls?: {
    sessionCount: number
    handshakeCount: number
    errorCount: number
    tlsVersion: string
    certificateStatus: string
    certificateExpireDays: number
    certificateIssuer: string
  } | null
  recentLogs?: Array<{
    level: string
    source: string
    message: string
    timestamp: number
  }>
  recentRequests?: HttpRequestRecord[]
}

/** Recent monitoring activity. */
export interface RecentActivity {
  id: string
  title: string
  category: string
  timestamp: number
}

/** Dashboard overview block. */
export interface OverviewStatistics {
  tunnelCount: number
  runningTunnel: number
  currentConnection: number
  todayTraffic: number
  totalTraffic: number
  averageRttMs: number
  runtimeUptimeSeconds: number
  healthScore: number
}

/** Unified dashboard payload consumed by Vue components. */
export interface DashboardData {
  overview: OverviewStatistics
  statistics: Statistics
  realtimeSpeed: RealtimeSpeedPoint[]
  connectionTrend: ConnectionTrendPoint[]
  trafficTrend: TrafficTrendPoint[]
  tunnelStatus: Array<{ label: string; count: number }>
  serverStatus: Array<{ label: string; count: number }>
  systemHealth: HealthReport
  tunnels: DashboardTunnel[]
  recentActivity: RecentActivity[]
  generatedAt: number
}
