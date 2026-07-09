/* ==================================================================
   Server 模块类型系统
   ------------------------------------------------------------------
   Server 是整个系统的资源管理中心。所有 Tunnel 必须绑定一个 Server。
   一个客户端可管理多个 Server（个人服务器 / 云服务器 / 家庭 NAS /
   公司服务器 / Docker 部署 / Kubernetes 未来扩展）。

   设计原则：
     - 类型契约与 Mock 解耦，未来替换为真实 Rust Server 时
       仅需保持类型不变即可无缝迁移 UI。
     - Server 不是配置，是资源。用户管理 Server Cluster。
   ================================================================== */

/* ── 连接状态（统一全模块） ── */
export type ServerStatus =
  | 'connected' // 已连接
  | 'disconnected' // 已断开
  | 'connecting' // 连接中
  | 'reconnecting' // 重连中
  | 'offline' // 离线
  | 'maintenance' // 维护中
  | 'error' // 错误

/* ── 服务器类型 ──
   V1 启用：personal / cloud / nas / company / docker
   未来扩展：kubernetes */
export type ServerKind =
  | 'personal' // 个人服务器
  | 'cloud' // 云服务器
  | 'nas' // 家庭 NAS
  | 'company' // 公司服务器
  | 'docker' // Docker 部署
  | 'kubernetes' // Kubernetes（未来）

/* ── 类型可用性 ── */
export type KindAvailability = 'enabled' | 'soon' | 'planned'

/* ── 连接方式 ── */
export type ConnectionMethod = 'tcp' | 'ws' | 'wss' | 'grpc'

/* ── 资源采样点（Mini Chart） ── */
export interface ServerMonitorPoint {
  time: string // 时间标签
  cpu: number // 百分比 0-100
  memory: number // 百分比 0-100
  network: number // bytes/s
}

/* ── 资源监控 ── */
export interface ServerMonitor {
  cpu: ServerResourceMetric
  memory: ServerResourceMetric
  disk: ServerResourceMetric
  load: ServerLoadMetric
  network: ServerNetworkMetric
  traffic: ServerTraffic
  connections: ServerConnectionMetric
}

/* ── 资源指标（百分比） ── */
export interface ServerResourceMetric {
  percent: number // 当前百分比 0-100
  used: number // 已用（GB / MB）
  total: number // 总量（GB / MB）
  unit: 'GB' | 'MB'
  history: number[] // 百分比采样（Mock）
}

/* ── 负载指标 ── */
export interface ServerLoadMetric {
  load1: number // 1 分钟负载
  load5: number // 5 分钟负载
  load15: number // 15 分钟负载
  cores: number // CPU 核心数
}

/* ── 网络指标 ── */
export interface ServerNetworkMetric {
  uploadSpeed: number // bytes/s
  downloadSpeed: number // bytes/s
  totalUpload: number // 累计 bytes
  totalDownload: number // 累计 bytes
  history: ServerMonitorPoint[]
}

/* ── 连接指标 ── */
export interface ServerConnectionMetric {
  active: number // 当前活动连接
  total: number // 累计连接
  failed: number // 失败连接
}

/* ── 流量采样点 ── */
export interface ServerTrafficPoint {
  time: string
  upload: number // bytes/s
  download: number // bytes/s
}

/* ── 流量统计 ── */
export interface ServerTraffic {
  uploadSpeed: number // 当前上传 bytes/s
  downloadSpeed: number // 当前下载 bytes/s
  totalUpload: number // 累计上传 bytes
  totalDownload: number // 累计下载 bytes
  todayUpload: number
  todayDownload: number
  weekUpload: number
  weekDownload: number
  monthUpload: number
  monthDownload: number
  history: ServerTrafficPoint[]
}

/* ── 单条连接 ── */
export interface ServerConnection {
  id: string
  clientIp: string
  region: string
  duration: number // 秒
  status: 'active' | 'idle' | 'closed'
  protocol: string
  startedAt: string // ISO
}

/* ── 日志等级 ── */
export type ServerLogLevel = 'debug' | 'info' | 'warn' | 'error' | 'success'

/* ── 单条日志 ── */
export interface ServerLog {
  id: string
  level: ServerLogLevel
  message: string
  timestamp: number // epoch ms
  source: string // 来源标识，如 "gateway" / "transport" / "auth" / "tunnel"
}

/* ── 健康检查项 ── */
export type HealthItemStatus = 'pass' | 'warn' | 'fail' | 'pending'

/* ── 健康检查项 ── */
export interface ServerHealthItem {
  key: string
  label: string
  status: HealthItemStatus
  message: string
  latency: number // ms
  icon: string
}

/* ── 健康报告 ── */
export interface ServerHealth {
  overall: 'healthy' | 'warning' | 'critical' | 'unknown'
  score: number // 0-100
  checkedAt: number // epoch ms
  items: ServerHealthItem[]
}

/* ── 运行统计 ── */
export interface ServerStatistics {
  uptime: number // 运行时长秒
  tunnelCount: number // Tunnel 数
  projectCount: number // Project 数
  totalConnections: number // 累计连接数
  requests: number // 累计请求数
  avgPing: number // 平均 ping ms
  peakSpeed: number // 峰值速度 bytes/s
}

/* ── 服务器设置 ── */
export interface ServerSettings {
  name: string
  host: string
  port: number
  token: string
  remark: string
  heartbeatInterval: number // 心跳间隔 秒
  reconnectInterval: number // 重连间隔 秒
  autoConnect: boolean
}

/* ── 服务器概览信息 ── */
export interface ServerOverviewInfo {
  hostname: string
  os: string
  arch: string
  rustVersion: string // Mock
  serverVersion: string
  installTime: string // ISO
  lastOnline: string // 人类可读
  lastHeartbeat: string // 人类可读
}

/* ── Server 核心实体 ── */
export interface Server {
  id: string
  name: string
  kind: ServerKind
  region: string
  publicIp: string
  version: string
  status: ServerStatus
  connectionMethod: ConnectionMethod
  ping: number // ms
  tags: string[]
  favorite: boolean
  recent: boolean
  overview: ServerOverviewInfo
  monitor: ServerMonitor
  traffic: ServerTraffic
  statistics: ServerStatistics
  connections: ServerConnection[]
  logs: ServerLog[]
  health: ServerHealth
  settings: ServerSettings
  lastConnectedAt: string // 人类可读相对时间
  createdAt: string // ISO
  updatedAt: string // ISO
}

/* ── 筛选类型 ── */
export type ServerFilterType = 'all' | 'online' | 'offline' | 'favorite' | 'recent' | 'unhealthy'

/* ── 排序类型 ── */
export type ServerSortType = 'name' | 'ping' | 'cpu' | 'memory' | 'tunnels' | 'projects' | 'region'

/* ── 排序方向 ── */
export type SortDirection = 'asc' | 'desc'

/* ── 工作区标签页 ── */
export type ServerWorkspaceTab =
  | 'overview'
  | 'monitor'
  | 'health'
  | 'tunnels'
  | 'projects'
  | 'traffic'
  | 'logs'
  | 'statistics'
  | 'settings'

/* ── 表单数据（创建 / 编辑） ── */
export interface ServerFormData {
  name: string
  kind: ServerKind
  host: string
  port: number | null
  token: string
  region: string
  remark: string
  tags: string[]
  heartbeatInterval: number
  reconnectInterval: number
  autoConnect: boolean
}

/* ── 类型预设 ── */
export interface KindPreset {
  key: ServerKind
  label: string
  description: string
  availability: KindAvailability
  icon: string
  color: string
}

/* ── 预置标签 ── */
export interface TagPreset {
  name: string
  color: string
}

/* ── 加载状态 ── */
export type ServerLoadStatus = 'idle' | 'loading' | 'success' | 'error'

/* ── 状态配置 ── */
export interface ServerStatusConfig {
  label: string
  dotStatus: 'online' | 'offline' | 'connecting' | 'error' | 'warning'
  badgeVariant: 'success' | 'warning' | 'error' | 'neutral' | 'info'
  pulse: boolean
  weight: number // 排序权重，越小越靠前
}
