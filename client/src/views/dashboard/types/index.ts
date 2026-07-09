/* ==================================================================
   Dashboard 类型系统
   ------------------------------------------------------------------
   所有 Dashboard 业务实体的类型定义。后续替换为真实接口时，
   仅需保持类型契约不变即可无缝迁移。
   ================================================================== */

// ── 运行态 ──
export type TunnelStatus = 'online' | 'offline' | 'connecting' | 'starting' | 'error'

export type ServerStatus = 'online' | 'offline' | 'connecting'

export type Protocol = 'tcp' | 'udp' | 'http' | 'https'

export type ActivityType = 'create' | 'start' | 'stop' | 'config' | 'connect' | 'update' | 'delete'

// ── 项目 ──
export interface DashboardProject {
  id: string
  name: string
  icon: string
  description: string
  tunnelCount: number
  runningCount: number
  lastStartedAt: string
  pinned: boolean
  favorite: boolean
  status: TunnelStatus
}

// ── 隧道 ──
export interface DashboardTunnel {
  id: string
  name: string
  protocol: Protocol
  status: TunnelStatus
  localPort: number
  publicPort: number
  publicHost: string
  uploadSpeed: number // KB/s
  downloadSpeed: number // KB/s
  connections: number
  projectId?: string
  https?: DashboardHttpsTunnel
}

export type CertificateStatus = 'active' | 'expiring_soon' | 'expired' | 'missing' | 'error'

export interface DashboardHttpsTunnel {
  certificateStatus: CertificateStatus
  expireDays: number
  issuer: string
  tlsVersion: string
  handshakeCount: number
  httpsTraffic: number // bytes
  errorCount: number
}

// ── 服务器 ──
export interface DashboardServer {
  id: string
  name: string
  region: string
  ip: string
  version: string
  ping: number // ms
  cpu: number // 0-100
  memory: number // 0-100
  disk: number // 0-100
  network: number // 0-100
  status: ServerStatus
  connected: boolean
}

// ── 活动 ──
export interface DashboardActivity {
  id: string
  type: ActivityType
  title: string
  description?: string
  timestamp: number // epoch ms
}

// ── 统计 ──
export interface DashboardStatistics {
  projectCount: number
  tunnelCount: number
  runningTunnel: number
  todayUpload: number // bytes
  todayDownload: number // bytes
  onlineTime: number // seconds
}

// ── 资源监控 ──
export interface DashboardResource {
  cpu: number // 0-100
  memory: number // 0-100
  traffic: number // 0-100
  connection: number // 0-100
}

// ── 资讯 ──
export interface DashboardNews {
  id: string
  type: 'release' | 'github' | 'changelog'
  title: string
  version?: string
  date: string
  summary: string
  url?: string
}

// ── 快捷操作 ──
export interface DashboardQuickAction {
  id: string
  icon: string
  label: string
  shortcut: string
  variant: 'primary' | 'success' | 'warning' | 'info'
}

// ── 加载状态 ──
export type DashboardLoadStatus = 'idle' | 'loading' | 'success' | 'error'
