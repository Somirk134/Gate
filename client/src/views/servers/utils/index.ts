/* ==================================================================
   Server 模块工具函数
   ------------------------------------------------------------------
   状态配置、类型预设、标签预设、格式化函数。
   统一从此处引入，禁止在组件中硬编码。
   ================================================================== */

import type {
  KindPreset,
  Server,
  ServerKind,
  ServerLogLevel,
  ServerStatus,
  ServerStatusConfig,
  TagPreset,
} from '../types'

/* ── 类型预设（V1 启用 personal/cloud/nas/company/docker，kubernetes 预留） ── */
export const KIND_PRESETS: KindPreset[] = [
  {
    key: 'personal',
    label: '个人',
    description: '个人服务器 / VPS',
    availability: 'enabled',
    icon: 'home',
    color: '#5B8DEF',
  },
  {
    key: 'cloud',
    label: '云服务器',
    description: '云服务器（阿里云 / 腾讯云 / AWS）',
    availability: 'enabled',
    icon: 'cloud',
    color: '#22C55E',
  },
  {
    key: 'nas',
    label: 'NAS',
    description: '家庭 NAS / 存储设备',
    availability: 'enabled',
    icon: 'hard-drive',
    color: '#F59E0B',
  },
  {
    key: 'company',
    label: '企业',
    description: '公司内网服务器',
    availability: 'enabled',
    icon: 'servers',
    color: '#7C6FF2',
  },
  {
    key: 'docker',
    label: 'Docker',
    description: 'Docker 容器部署',
    availability: 'enabled',
    icon: 'box',
    color: '#06B6D4',
  },
  {
    key: 'kubernetes',
    label: 'Kubernetes',
    description: 'K8s 集群（即将支持）',
    availability: 'soon',
    icon: 'layers',
    color: '#EF4444',
  },
]

/* 类型键 → 预设映射 */
export const KIND_MAP: Record<ServerKind, KindPreset> = KIND_PRESETS.reduce(
  (acc, p) => {
    acc[p.key] = p
    return acc
  },
  {} as Record<ServerKind, KindPreset>,
)

/* ── 预置标签 ── */
export const SERVER_TAGS: TagPreset[] = [
  { name: '生产', color: '#EF4444' },
  { name: '开发', color: '#06B6D4' },
  { name: '测试', color: '#F59E0B' },
  { name: '家庭', color: '#5B8DEF' },
  { name: '云服务', color: '#22C55E' },
  { name: '阿里云', color: '#F59E0B' },
  { name: '腾讯云', color: '#5B8DEF' },
  { name: 'AWS', color: '#EF4444' },
]

/* ── 状态配置：统一全模块状态展示 ── */
export const SERVER_STATUS_CONFIG: Record<ServerStatus, ServerStatusConfig> = {
  connected: {
    label: '已连接',
    dotStatus: 'online',
    badgeVariant: 'success',
    pulse: false,
    weight: 0,
  },
  connecting: {
    label: '连接中',
    dotStatus: 'connecting',
    badgeVariant: 'info',
    pulse: true,
    weight: 1,
  },
  reconnecting: {
    label: '重连中',
    dotStatus: 'connecting',
    badgeVariant: 'warning',
    pulse: true,
    weight: 2,
  },
  maintenance: {
    label: '维护中',
    dotStatus: 'warning',
    badgeVariant: 'warning',
    pulse: false,
    weight: 3,
  },
  error: {
    label: '错误',
    dotStatus: 'error',
    badgeVariant: 'error',
    pulse: false,
    weight: 4,
  },
  disconnected: {
    label: '已断开',
    dotStatus: 'warning',
    badgeVariant: 'warning',
    pulse: false,
    weight: 5,
  },
  offline: {
    label: '离线',
    dotStatus: 'offline',
    badgeVariant: 'neutral',
    pulse: false,
    weight: 6,
  },
}

/* 状态排序权重 */
export const STATUS_WEIGHT: Record<ServerStatus, number> = Object.fromEntries(
  (Object.entries(SERVER_STATUS_CONFIG) as [ServerStatus, ServerStatusConfig][]).map(([k, v]) => [
    k,
    v.weight,
  ]),
) as Record<ServerStatus, number>

/* ── 状态是否处于在线语义 ── */
export function isOnlineStatus(status: ServerStatus): boolean {
  return status === 'connected'
}

/* ── 状态是否处于过渡态（不可点击操作） ── */
export function isTransitionStatus(status: ServerStatus): boolean {
  return status === 'connecting' || status === 'reconnecting'
}

/* ── hex → rgba ── */
function hexToRgba(hex: string, alpha: number): string {
  const h = hex.replace('#', '')
  const r = parseInt(h.substring(0, 2), 16)
  const g = parseInt(h.substring(2, 4), 16)
  const b = parseInt(h.substring(4, 6), 16)
  return `rgba(${r}, ${g}, ${b}, ${alpha})`
}

/**
 * 生成类型颜色 CSS 变量对象，注入到卡片 / 头部：
 *   --server-color        实色
 *   --server-color-muted  15% 透明度底色
 *   --server-color-soft   8% 透明度底色
 */
export function serverColorVars(kind: ServerKind): Record<string, string> {
  const hex = KIND_MAP[kind]?.color ?? '#5B8DEF'
  return {
    '--server-color': hex,
    '--server-color-muted': hexToRgba(hex, 0.15),
    '--server-color-soft': hexToRgba(hex, 0.08),
  }
}

/* ── 格式化：字节 ── */
export function formatBytes(bytes: number): string {
  if (bytes <= 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  const i = Math.floor(Math.log(bytes) / Math.log(1024))
  const v = bytes / Math.pow(1024, i)
  return `${v.toFixed(v < 10 && i > 0 ? 1 : 0)} ${units[i]}`
}

/* ── 格式化：速度（bytes/s） ── */
export function formatSpeed(bytesPerSec: number): string {
  if (bytesPerSec <= 0) return '0 B/s'
  return `${formatBytes(bytesPerSec)}/s`
}

/* ── 格式化：时长（秒） ── */
export function formatDuration(seconds: number): string {
  if (seconds <= 0) return '0 秒'
  const d = Math.floor(seconds / 86400)
  const h = Math.floor((seconds % 86400) / 3600)
  const m = Math.floor((seconds % 3600) / 60)
  const s = Math.floor(seconds % 60)
  if (d > 0) return `${d} 天 ${h} 小时`
  if (h > 0) return `${h} 小时 ${m} 分钟`
  if (m > 0) return `${m} 分钟 ${s} 秒`
  return `${s} 秒`
}

/* ── 格式化：数字（千分位） ── */
export function formatNumber(n: number): string {
  return n.toLocaleString('zh-CN')
}

/* ── 格式化：日期时间 ── */
export function formatDateTime(iso: string): string {
  const d = new Date(iso)
  const pad = (n: number) => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())} ${pad(d.getHours())}:${pad(d.getMinutes())}`
}

/* ── 格式化：仅日期 ── */
export function formatDate(iso: string): string {
  const d = new Date(iso)
  const pad = (n: number) => String(n).padStart(2, '0')
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}`
}

/* ── 格式化：日志时间戳 ── */
export function formatLogTime(ts: number): string {
  const d = new Date(ts)
  const pad = (n: number) => String(n).padStart(2, '0')
  return `${pad(d.getHours())}:${pad(d.getMinutes())}:${pad(d.getSeconds())}.${String(d.getMilliseconds()).padStart(3, '0')}`
}

/* ── 生成唯一 ID ── */
export function genId(prefix = 's'): string {
  return `${prefix}-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 7)}`
}

/* ── 端口校验 ── */
export function isValidPort(port: number | null): boolean {
  return port != null && port >= 1 && port <= 65535
}

/* ── IP 校验 ── */
export function isValidIp(ip: string): boolean {
  const v4 = /^(\d{1,3}\.){3}\d{1,3}$/
  if (!v4.test(ip)) return false
  return ip.split('.').every((p) => {
    const n = Number(p)
    return n >= 0 && n <= 255
  })
}

/* ── 主机校验（IP 或域名） ── */
export function isValidHost(host: string): boolean {
  if (!host.trim()) return false
  if (isValidIp(host)) return true
  // 域名
  return /^[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?(\.[a-zA-Z0-9]([a-zA-Z0-9-]{0,61}[a-zA-Z0-9])?)+$/.test(
    host,
  )
}

/* ── Token 校验 ── */
export function isValidToken(token: string): boolean {
  return token.trim().length >= 8
}

/* ── 健康检查得分 ── */
export function healthScore(server: Server): number {
  return server.health.score
}

/* ── 健康状态颜色 ── */
export function healthColor(overall: Server['health']['overall']): string {
  switch (overall) {
    case 'healthy':
      return '#22C55E'
    case 'warning':
      return '#F59E0B'
    case 'critical':
      return '#EF4444'
    default:
      return '#6B6B72'
  }
}

/* ── 取 Server 累计流量 ── */
export function totalTraffic(s: Server): number {
  return s.traffic.totalUpload + s.traffic.totalDownload
}

/* ── 推送日志工具（store 内部复用） ── */
export function makeLog(
  level: ServerLogLevel,
  message: string,
  source: string,
): { id: string; level: ServerLogLevel; message: string; timestamp: number; source: string } {
  return {
    id: genId('slog'),
    level,
    message,
    timestamp: Date.now(),
    source,
  }
}
