/* ==================================================================
   Tunnel 模块类型系统
   ------------------------------------------------------------------
   Tunnel 是整个软件最核心的业务对象，拥有完整的生命周期、状态、
   统计、日志、连接、配置与监控。

   设计原则：
     - 所有类型契约与 Mock 解耦，未来替换为真实 Tunnel Engine 时
       仅需保持类型不变即可无缝迁移 UI。
     - 协议字段已预留 HTTPS / UDP / P2P 扩展，V1 仅启用 HTTP / TCP。
   ================================================================== */

/* ── 运行状态（统一全模块） ── */
export type TunnelStatus =
  | "running" // 运行中
  | "stopped" // 已停止
  | "starting" // 启动中
  | "stopping" // 停止中
  | "restarting" // 重启中
  | "error" // 异常
  | "disconnected" // 已断开
  | "connecting" // 连接中
  | "offline" // 离线

/* ── 协议 ──
   V1 启用：http / tcp
   未来扩展：https / udp / p2p（界面已预留） */
export type TunnelProtocol = "http" | "tcp" | "https" | "udp" | "p2p"

/* ── 协议可用性 ── */
export type ProtocolAvailability = "enabled" | "soon" | "planned"

/* ── 流量采样点（Mini Chart） ── */
export interface TunnelTrafficPoint {
  time: string // 时间标签
  upload: number // bytes/s
  download: number // bytes/s
}

/* ── 流量统计 ── */
export interface TunnelTraffic {
  uploadSpeed: number // 当前上传速度 bytes/s
  downloadSpeed: number // 当前下载速度 bytes/s
  totalUpload: number // 累计上传 bytes
  totalDownload: number // 累计下载 bytes
  todayUpload: number // 今日上传 bytes
  todayDownload: number // 今日下载 bytes
  history: TunnelTrafficPoint[] // 历史采样（Mock）
}

/* ── 单条连接 ── */
export interface TunnelConnection {
  id: string
  clientIp: string
  region: string
  duration: number // 已持续秒数
  status: "active" | "idle" | "closed"
  protocol: TunnelProtocol
  startedAt: string // ISO
}

/* ── 日志等级 ── */
export type TunnelLogLevel = "debug" | "info" | "warn" | "error" | "success"

/* ── 单条日志 ── */
export interface TunnelLog {
  id: string
  level: TunnelLogLevel
  message: string
  timestamp: number // epoch ms
  source: string // 来源标识，如 "frpc" / "transport" / "api"
}

/* ── 运行统计 ── */
export interface TunnelStatistics {
  uptime: number // 运行时长秒
  connections: number // 当前连接数
  totalConnections: number // 累计连接数
  requests: number // 累计请求数
  avgLatency: number // 平均延迟 ms
  peakSpeed: number // 峰值速度 bytes/s
}

/* ── 隧道设置（预留压缩 / 加密） ── */
export interface TunnelSettings {
  compression: boolean // 预留
  encryption: boolean // 预留
}

/* ── Tunnel 核心实体 ── */
export interface Tunnel {
  id: string
  name: string
  protocol: TunnelProtocol
  localHost: string
  localPort: number
  remotePort: number
  publicAddr: string // 公网访问地址
  remark: string
  status: TunnelStatus
  autoStart: boolean
  compression: boolean // 预留
  encryption: boolean // 预留
  tags: string[]
  serverName: string
  projectName: string
  projectId: string
  pinned: boolean
  favorite: boolean
  traffic: TunnelTraffic
  statistics: TunnelStatistics
  connections: TunnelConnection[]
  logs: TunnelLog[]
  lastStartedAt: string // 人类可读相对时间
  createdAt: string // ISO
  updatedAt: string // ISO
}

/* ── 筛选类型 ── */
export type TunnelFilterType =
  | "all"
  | "http"
  | "tcp"
  | "running"
  | "stopped"
  | "favorite"
  | "recent"

/* ── 排序类型 ── */
export type TunnelSortType =
  | "name"
  | "status"
  | "traffic"
  | "connections"
  | "createdAt"
  | "updatedAt"

/* ── 排序方向 ── */
export type SortDirection = "asc" | "desc"

/* ── 工作区标签页 ── */
export type TunnelWorkspaceTab =
  | "overview"
  | "traffic"
  | "connection"
  | "logs"
  | "statistics"
  | "settings"
  | "monitor"

/* ── 表单数据（创建 / 编辑） ── */
export interface TunnelFormData {
  name: string
  protocol: TunnelProtocol
  localHost: string
  localPort: number | null
  remotePort: number | null
  projectId: string
  serverName: string
  autoStart: boolean
  remark: string
  tags: string[]
}

/* ── 协议预设 ── */
export interface ProtocolPreset {
  key: TunnelProtocol
  label: string
  description: string
  availability: ProtocolAvailability
  icon: string
  color: string
}

/* ── 预置标签 ── */
export interface TagPreset {
  name: string
  color: string
}

/* ── 加载状态 ── */
export type TunnelLoadStatus = "idle" | "loading" | "success" | "error"

/* ── 状态配置 ── */
export interface TunnelStatusConfig {
  label: string
  dotStatus: "online" | "offline" | "connecting" | "starting" | "error" | "warning"
  badgeVariant: "success" | "warning" | "error" | "neutral" | "info"
  pulse: boolean
  weight: number // 排序权重，越小越靠前
}
