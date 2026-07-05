/* ==================================================================
   Project 模块类型系统
   ------------------------------------------------------------------
   所有 Project 业务实体的类型定义。后续替换为真实接口时，
   仅需保持类型契约不变即可无缝迁移。
   ================================================================== */

// ── 运行状态 ──
export type ProjectStatus =
  | "running" // 运行中（全部 Tunnel 在线）
  | "partial" // 部分运行（部分 Tunnel 在线）
  | "stopped" // 已停止（无 Tunnel 在线）
  | "starting" // 启动中
  | "error" // 异常

// ── 颜色预设键 ──
export type ProjectColor =
  | "blue"
  | "green"
  | "purple"
  | "orange"
  | "red"
  | "cyan"
  | "pink"
  | "indigo"
  | "teal"
  | "amber"
  | "slate"

// ── 标签 ──
export interface ProjectTag {
  id: string
  name: string
}

// ── 项目统计 ──
export interface ProjectStatistics {
  todayTraffic: number // bytes
  totalTraffic: number // bytes
  uptime: number // seconds
  connections: number
  tunnelCount: number
  runningTunnelCount: number
}

// ── 项目核心接口 ──
export interface Project {
  id: string
  name: string
  description: string
  icon: string // Lucide 图标名
  color: ProjectColor
  tags: string[]
  serverName: string
  autoStart: boolean
  remark?: string // 备注
  status: ProjectStatus
  pinned: boolean
  favorite: boolean
  lastUsedAt: number // epoch ms，最近使用时间
  tunnelCount: number
  runningTunnelCount: number
  statistics: ProjectStatistics
  lastStartedAt: string // 人类可读的相对时间
  createdAt: string // ISO 时间字符串
  updatedAt: string // ISO 时间字符串
}

// ── 筛选类型 ──
export type ProjectFilterType =
  | "all"
  | "running"
  | "stopped"
  | "favorite"
  | "recent"

// ── 排序类型 ──
export type ProjectSortType =
  | "name"
  | "createdAt"
  | "updatedAt"
  | "status"
  | "tunnelCount"

// ── 排序方向 ──
export type SortDirection = "asc" | "desc"

// ── 表单数据 ──
export interface ProjectFormData {
  name: string
  icon: string
  color: ProjectColor
  description: string
  serverName: string
  autoStart: boolean
  tags: string[]
  remark: string
}

// ── 颜色预设 ──
export interface ColorPreset {
  key: ProjectColor
  label: string
  value: string // hex
}

// ── 图标预设 ──
export interface IconPreset {
  key: string
  label: string
}

// ── 预置标签 ──
export interface TagPreset {
  name: string
  color: string
}

// ── 加载状态 ──
export type ProjectLoadStatus = "idle" | "loading" | "success" | "error"

// ── Mock Tunnel（详情页占位，不开发真实 Tunnel） ──
export interface MockTunnel {
  id: string
  name: string
  protocol: "tcp" | "http" | "https"
  localAddr: string
  remoteAddr: string
  publicAddr: string
  status: "online" | "offline" | "starting" | "error"
  downSpeed: string
  upSpeed: string
  connections: number
}
