/* ==================================================================
   Tunnel 模块工具函数
   ------------------------------------------------------------------
   状态配置、协议预设、标签预设、格式化函数。
   统一从此处引入，禁止在组件中硬编码。
   ================================================================== */

import type {
  ProtocolPreset,
  TagPreset,
  Tunnel,
  TunnelProtocol,
  TunnelStatus,
  TunnelStatusConfig,
} from '../types'

/* ── 协议预设（V1 启用 http/tcp，其余预留扩展） ── */
export const PROTOCOL_PRESETS: ProtocolPreset[] = [
  {
    key: 'http',
    label: 'HTTP',
    description: 'Web 服务 / API 反向代理',
    availability: 'enabled',
    icon: 'globe',
    color: '#5B8DEF',
  },
  {
    key: 'tcp',
    label: 'TCP',
    description: '任意 TCP 端口转发',
    availability: 'enabled',
    icon: 'router',
    color: '#22C55E',
  },
  {
    key: 'https',
    label: 'HTTPS',
    description: '加密 Web 反向代理，自动证书与 SNI 路由',
    availability: 'enabled',
    icon: 'shield',
    color: '#7C6FF2',
  },
  {
    key: 'udp',
    label: 'UDP',
    description: 'UDP 协议转发（计划中）',
    availability: 'planned',
    icon: 'radio',
    color: '#F59E0B',
  },
  {
    key: 'p2p',
    label: 'P2P',
    description: '点对点穿透（计划中）',
    availability: 'planned',
    icon: 'network',
    color: '#06B6D4',
  },
]

/* 协议键 → 预设映射 */
export const PROTOCOL_MAP: Record<TunnelProtocol, ProtocolPreset> = PROTOCOL_PRESETS.reduce(
  (acc, p) => {
    acc[p.key] = p
    return acc
  },
  {} as Record<TunnelProtocol, ProtocolPreset>,
)

/* ── 预置标签 ── */
export const TUNNEL_TAGS: TagPreset[] = [
  { name: 'API', color: '#5B8DEF' },
  { name: 'Frontend', color: '#06B6D4' },
  { name: 'SSH', color: '#22C55E' },
  { name: 'Database', color: '#EF4444' },
  { name: 'Demo', color: '#F59E0B' },
  { name: 'Production', color: '#EF4444' },
  { name: 'Staging', color: '#06B6D4' },
  { name: 'Personal', color: '#7C6FF2' },
]

/* ── 状态配置：统一全模块状态展示 ── */
export const TUNNEL_STATUS_CONFIG: Record<TunnelStatus, TunnelStatusConfig> = {
  running: {
    label: '运行中',
    dotStatus: 'online',
    badgeVariant: 'success',
    pulse: false,
    weight: 0,
  },
  connecting: {
    label: '连接中',
    dotStatus: 'connecting',
    badgeVariant: 'warning',
    pulse: true,
    weight: 1,
  },
  starting: {
    label: '启动中',
    dotStatus: 'starting',
    badgeVariant: 'info',
    pulse: true,
    weight: 2,
  },
  restarting: {
    label: '重启中',
    dotStatus: 'connecting',
    badgeVariant: 'info',
    pulse: true,
    weight: 3,
  },
  stopping: {
    label: '停止中',
    dotStatus: 'warning',
    badgeVariant: 'warning',
    pulse: true,
    weight: 4,
  },
  error: {
    label: '异常',
    dotStatus: 'error',
    badgeVariant: 'error',
    pulse: false,
    weight: 5,
  },
  disconnected: {
    label: '已断开',
    dotStatus: 'warning',
    badgeVariant: 'warning',
    pulse: false,
    weight: 6,
  },
  stopped: {
    label: '已停止',
    dotStatus: 'offline',
    badgeVariant: 'neutral',
    pulse: false,
    weight: 7,
  },
  offline: {
    label: '离线',
    dotStatus: 'offline',
    badgeVariant: 'neutral',
    pulse: false,
    weight: 8,
  },
}

/* 状态排序权重（供排序复用） */
export const STATUS_WEIGHT: Record<TunnelStatus, number> = Object.fromEntries(
  (Object.entries(TUNNEL_STATUS_CONFIG) as [TunnelStatus, TunnelStatusConfig][]).map(([k, v]) => [
    k,
    v.weight,
  ]),
) as Record<TunnelStatus, number>

/* ── 状态是否处于运行中语义 ── */
export function isRunningStatus(status: TunnelStatus): boolean {
  return (
    status === 'running' ||
    status === 'starting' ||
    status === 'connecting' ||
    status === 'restarting'
  )
}

/* ── 状态是否处于过渡态（不可点击操作） ── */
export function isTransitionStatus(status: TunnelStatus): boolean {
  return (
    status === 'starting' ||
    status === 'stopping' ||
    status === 'restarting' ||
    status === 'connecting'
  )
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
 * 生成协议颜色 CSS 变量对象，注入到卡片 / 头部：
 *   --tunnel-color        实色
 *   --tunnel-color-muted  15% 透明度底色
 *   --tunnel-color-soft   8% 透明度底色
 */
export function tunnelColorVars(protocol: TunnelProtocol): Record<string, string> {
  const hex = PROTOCOL_MAP[protocol]?.color ?? '#5B8DEF'
  return {
    '--tunnel-color': hex,
    '--tunnel-color-muted': hexToRgba(hex, 0.15),
    '--tunnel-color-soft': hexToRgba(hex, 0.08),
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
export function genId(prefix = 't'): string {
  return `${prefix}-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 7)}`
}

/* ── 端口校验 ── */
export function isValidPort(port: number | null): boolean {
  return port != null && port >= 1 && port <= 65535
}

/* ── 公网地址拼接 ── */
export function buildPublicAddr(serverDomain: string, remotePort: number): string {
  return `${serverDomain}:${remotePort}`
}

/* ── 取 Tunnel 主流量（上传+下载累计） ── */
export function totalTraffic(t: Tunnel): number {
  return t.traffic.totalUpload + t.traffic.totalDownload
}
