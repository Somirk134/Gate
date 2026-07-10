/* ==================================================================
   Server 模块工具函数
   ------------------------------------------------------------------
   状态配置、类型预设、标签预设、格式化函数。
   统一从此处引入，禁止在组件中硬编码。
   ================================================================== */

import type {
  KindPreset,
  ServerKind,
  ServerStatus,
  ServerStatusConfig,
  TagPreset,
} from '../types'

/* ── 类型预设（V1 启用 personal/cloud/nas/company/docker） ── */
export const KIND_PRESETS: KindPreset[] = [
  {
    key: 'personal',
    label: 'personal',
    description: 'personal',
    availability: 'enabled',
    icon: 'home',
    color: '#5B8DEF',
  },
  {
    key: 'cloud',
    label: 'cloud',
    description: 'cloud',
    availability: 'enabled',
    icon: 'cloud',
    color: '#22C55E',
  },
  {
    key: 'nas',
    label: 'NAS',
    description: 'nas',
    availability: 'enabled',
    icon: 'hard-drive',
    color: '#F59E0B',
  },
  {
    key: 'company',
    label: 'company',
    description: 'company',
    availability: 'enabled',
    icon: 'servers',
    color: '#7C6FF2',
  },
  {
    key: 'docker',
    label: 'Docker',
    description: 'docker',
    availability: 'enabled',
    icon: 'box',
    color: '#06B6D4',
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
  { name: 'production', color: '#EF4444' },
  { name: 'development', color: '#06B6D4' },
  { name: 'testing', color: '#F59E0B' },
  { name: 'home', color: '#5B8DEF' },
  { name: 'cloudService', color: '#22C55E' },
  { name: 'aliyun', color: '#F59E0B' },
  { name: 'tencentCloud', color: '#5B8DEF' },
  { name: 'aws', color: '#EF4444' },
]

/* ── 状态配置：统一全模块状态展示 ── */
export const SERVER_STATUS_CONFIG: Record<ServerStatus, ServerStatusConfig> = {
  connected: {
    label: 'connected',
    dotStatus: 'online',
    badgeVariant: 'success',
    pulse: false,
    weight: 0,
  },
  connecting: {
    label: 'connecting',
    dotStatus: 'connecting',
    badgeVariant: 'info',
    pulse: true,
    weight: 1,
  },
  reconnecting: {
    label: 'reconnecting',
    dotStatus: 'connecting',
    badgeVariant: 'warning',
    pulse: true,
    weight: 2,
  },
  maintenance: {
    label: 'maintenance',
    dotStatus: 'warning',
    badgeVariant: 'warning',
    pulse: false,
    weight: 3,
  },
  error: {
    label: 'error',
    dotStatus: 'error',
    badgeVariant: 'error',
    pulse: false,
    weight: 4,
  },
  disconnected: {
    label: 'disconnected',
    dotStatus: 'warning',
    badgeVariant: 'warning',
    pulse: false,
    weight: 5,
  },
  offline: {
    label: 'offline',
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
type TranslateFn = (key: string, params?: Record<string, number>) => string

export function formatDuration(seconds: number, t: TranslateFn): string {
  if (seconds <= 0) return t('common.time.seconds', { count: 0 })
  const d = Math.floor(seconds / 86400)
  const h = Math.floor((seconds % 86400) / 3600)
  const m = Math.floor((seconds % 3600) / 60)
  const s = Math.floor(seconds % 60)
  if (d > 0) return t('common.time.daysHours', { days: d, hours: h })
  if (h > 0) return t('common.time.hoursMinutes', { hours: h, minutes: m })
  if (m > 0) return t('common.time.minutesSeconds', { minutes: m, seconds: s })
  return t('common.time.seconds', { count: s })
}

/* ── 格式化：数字（千分位） ── */
export function formatNumber(n: number): string {
  return n.toLocaleString('zh-CN')
}
