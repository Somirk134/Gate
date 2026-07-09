/* ==================================================================
   Project 模块工具函数
   ------------------------------------------------------------------
   颜色预设、图标预设、标签预设、状态配置、格式化函数。
   统一从此处引入，禁止在组件中硬编码。
   ================================================================== */

import type {
  ColorPreset,
  IconPreset,
  ProjectColor,
  ProjectTemplateProfile,
  ProjectStatus,
  ProjectTag,
  TagPreset,
} from '../types'

/* ── 颜色预设 ── */
export const PROJECT_COLORS: ColorPreset[] = [
  { key: 'blue', label: 'blue', value: '#5B8DEF' },
  { key: 'green', label: 'green', value: '#22C55E' },
  { key: 'purple', label: 'purple', value: '#7C6FF2' },
  { key: 'orange', label: 'orange', value: '#F59E0B' },
  { key: 'red', label: 'red', value: '#EF4444' },
  { key: 'cyan', label: 'cyan', value: '#06B6D4' },
  { key: 'pink', label: 'pink', value: '#EC4899' },
  { key: 'indigo', label: 'indigo', value: '#6366F1' },
  { key: 'teal', label: 'teal', value: '#14B8A6' },
  { key: 'amber', label: 'amber', value: '#D97706' },
  { key: 'slate', label: 'slate', value: '#64748B' },
]

/* 颜色键 → hex 映射，便于 O(1) 查找 */
export const COLOR_MAP: Record<ProjectColor, string> = PROJECT_COLORS.reduce(
  (acc, c) => {
    acc[c.key] = c.value
    return acc
  },
  {} as Record<ProjectColor, string>,
)

/* ── 图标预设（均为 registry 中已注册的 Lucide 图标） ── */
export const PROJECT_ICONS: IconPreset[] = [
  { key: 'package', label: 'project' },
  { key: 'globe', label: 'domain' },
  { key: 'database', label: 'database' },
  { key: 'servers', label: 'servers' },
  { key: 'cloud', label: 'cloud' },
  { key: 'code', label: 'code' },
  { key: 'box', label: 'container' },
  { key: 'terminal', label: 'terminal' },
  { key: 'activity', label: 'activity' },
  { key: 'cpu', label: 'CPU' },
  { key: 'router', label: 'router' },
  { key: 'layers', label: 'layers' },
  { key: 'boxes', label: 'resourceGroup' },
  { key: 'network', label: 'network' },
  { key: 'shield', label: 'security' },
  { key: 'zap', label: 'fast' },
  { key: 'rocket', label: 'launch' },
  { key: 'hard-drive', label: 'storage' },
  { key: 'link', label: 'link' },
  { key: 'plug', label: 'plugin' },
]

export const PROJECT_TEMPLATES: ProjectTemplateProfile[] = [
  {
    key: 'blank',
    label: 'blank',
    icon: 'package',
    color: 'blue',
    description: 'blank',
    tags: [],
    recommendations: [],
  },
  {
    key: 'springBoot',
    label: 'springBoot',
    icon: 'code',
    color: 'green',
    description: 'springBoot',
    tags: ['backend', 'java'],
    recommendations: [
      {
        id: 'spring-http',
        name: 'springHttp',
        protocol: 'http',
        localHost: '127.0.0.1',
        localPort: 8080,
        remotePort: 18080,
        description: 'springHttp',
        tags: ['backend', 'http'],
      },
    ],
  },
  {
    key: 'vue',
    label: 'vue',
    icon: 'code',
    color: 'cyan',
    description: 'vue',
    tags: ['frontend', 'vite'],
    recommendations: [
      {
        id: 'vue-dev',
        name: 'vueDev',
        protocol: 'http',
        localHost: '127.0.0.1',
        localPort: 5173,
        remotePort: 15173,
        description: 'vueDev',
        tags: ['frontend', 'http'],
      },
    ],
  },
  {
    key: 'node',
    label: 'node',
    icon: 'terminal',
    color: 'green',
    description: 'node',
    tags: ['node', 'api'],
    recommendations: [
      {
        id: 'node-api',
        name: 'nodeApi',
        protocol: 'http',
        localHost: '127.0.0.1',
        localPort: 3000,
        remotePort: 13000,
        description: 'nodeApi',
        tags: ['node', 'http'],
      },
    ],
  },
  {
    key: 'python',
    label: 'python',
    icon: 'file-code',
    color: 'blue',
    description: 'python',
    tags: ['python', 'api'],
    recommendations: [
      {
        id: 'python-api',
        name: 'pythonApi',
        protocol: 'http',
        localHost: '127.0.0.1',
        localPort: 8000,
        remotePort: 18000,
        description: 'pythonApi',
        tags: ['python', 'http'],
      },
    ],
  },
  {
    key: 'docker',
    label: 'docker',
    icon: 'box',
    color: 'indigo',
    description: 'docker',
    tags: ['docker'],
    recommendations: [
      {
        id: 'docker-web',
        name: 'dockerWeb',
        protocol: 'http',
        localHost: '127.0.0.1',
        localPort: 8080,
        remotePort: 18081,
        description: 'dockerWeb',
        tags: ['docker', 'http'],
      },
    ],
  },
  {
    key: 'mcpServer',
    label: 'mcpServer',
    icon: 'plug-zap',
    color: 'purple',
    description: 'mcpServer',
    tags: ['mcp', 'ai'],
    recommendations: [
      {
        id: 'mcp-server',
        name: 'mcpServer',
        protocol: 'http',
        localHost: '127.0.0.1',
        localPort: 3333,
        remotePort: 13333,
        description: 'mcpServer',
        tags: ['mcp', 'ai'],
      },
    ],
  },
  {
    key: 'aiAgent',
    label: 'AI Agent',
    icon: 'sparkles',
    color: 'pink',
    description: 'aiAgent',
    tags: ['ai', 'agent'],
    recommendations: [
      {
        id: 'agent-console',
        name: 'agentConsole',
        protocol: 'http',
        localHost: '127.0.0.1',
        localPort: 7860,
        remotePort: 17860,
        description: 'agentConsole',
        tags: ['ai', 'agent'],
      },
    ],
  },
  {
    key: 'nas',
    label: 'NAS',
    icon: 'hard-drive',
    color: 'slate',
    description: 'nas',
    tags: ['home', 'storage'],
    recommendations: [
      {
        id: 'nas-panel',
        name: 'nasPanel',
        protocol: 'http',
        localHost: '127.0.0.1',
        localPort: 5000,
        remotePort: 15000,
        description: 'nasPanel',
        tags: ['nas', 'home'],
      },
    ],
  },
  {
    key: 'ssh',
    label: 'SSH',
    icon: 'terminal',
    color: 'amber',
    description: 'ssh',
    tags: ['ssh'],
    recommendations: [
      {
        id: 'ssh',
        name: 'SSH',
        protocol: 'tcp',
        localHost: '127.0.0.1',
        localPort: 22,
        remotePort: 10022,
        description: 'ssh',
        tags: ['ssh', 'tcp'],
      },
    ],
  },
  {
    key: 'git',
    label: 'Git',
    icon: 'git-branch',
    color: 'orange',
    description: 'git',
    tags: ['git'],
    recommendations: [
      {
        id: 'git-http',
        name: 'Git HTTP',
        protocol: 'http',
        localHost: '127.0.0.1',
        localPort: 3000,
        remotePort: 13001,
        description: 'gitHttp',
        tags: ['git', 'http'],
      },
    ],
  },
  {
    key: 'webhook',
    label: 'Webhook',
    icon: 'radio',
    color: 'teal',
    description: 'webhook',
    tags: ['webhook'],
    recommendations: [
      {
        id: 'webhook',
        name: 'webhookReceiver',
        protocol: 'http',
        localHost: '127.0.0.1',
        localPort: 9000,
        remotePort: 19000,
        description: 'webhookReceiver',
        tags: ['webhook', 'http'],
      },
    ],
  },
  {
    key: 'custom',
    label: 'custom',
    icon: 'layers',
    color: 'blue',
    description: 'custom',
    tags: ['custom'],
    recommendations: [
      {
        id: 'custom-http',
        name: 'customHttp',
        protocol: 'http',
        localHost: '127.0.0.1',
        localPort: 8080,
        remotePort: 18082,
        description: 'customHttp',
        tags: ['custom'],
      },
    ],
  },
]

/* ── 预置标签 ── */
export const PROJECT_TAGS: TagPreset[] = [
  { name: 'work', color: '#5B8DEF' },
  { name: 'home', color: '#22C55E' },
  { name: 'demo', color: '#F59E0B' },
  { name: 'customer', color: '#EC4899' },
  { name: 'opensource', color: '#14B8A6' },
  { name: 'personal', color: '#7C6FF2' },
  { name: 'production', color: '#EF4444' },
  { name: 'staging', color: '#06B6D4' },
]

/* ── 状态配置：label + GBadge variant ── */
export const STATUS_CONFIG: Record<
  ProjectStatus,
  { label: string; variant: 'success' | 'warning' | 'error' | 'neutral' | 'info'; pulse: boolean }
> = {
  running: { label: 'running', variant: 'success', pulse: false },
  partial: { label: 'partial', variant: 'warning', pulse: false },
  stopped: { label: 'stopped', variant: 'neutral', pulse: false },
  starting: { label: 'starting', variant: 'info', pulse: true },
  error: { label: 'error', variant: 'error', pulse: false },
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
 * 生成项目颜色 CSS 变量对象，用于在元素上注入：
 *   --project-color        实色
 *   --project-color-muted  15% 透明度底色
 *   --project-color-soft   8% 透明度底色
 */
export function projectColorVars(color: ProjectColor): Record<string, string> {
  const hex = COLOR_MAP[color] ?? COLOR_MAP.blue
  return {
    '--project-color': hex,
    '--project-color-muted': hexToRgba(hex, 0.15),
    '--project-color-soft': hexToRgba(hex, 0.08),
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

/* ── 格式化：时长（秒） ── */
export function formatDuration(seconds: number): string {
  if (seconds <= 0) return '0s'
  const d = Math.floor(seconds / 86400)
  const h = Math.floor((seconds % 86400) / 3600)
  const m = Math.floor((seconds % 3600) / 60)
  const s = Math.floor(seconds % 60)
  if (d > 0) return `${d}d ${h}h`
  if (h > 0) return `${h}h ${m}m`
  if (m > 0) return `${m}m ${s}s`
  return `${s}s`
}

type TranslateFn = (key: string, params?: Record<string, number>) => string

export function formatRelativeTime(
  timestamp: number,
  t: TranslateFn,
  locale: string,
): string {
  if (!timestamp) return t('project.relativeTime.never')
  const diff = Date.now() - timestamp
  const minute = 60 * 1000
  const hour = 60 * minute
  const day = 24 * hour
  if (diff < minute) return t('project.relativeTime.justNow')
  if (diff < hour) return t('project.relativeTime.minutesAgo', { count: Math.floor(diff / minute) })
  if (diff < day) return t('project.relativeTime.hoursAgo', { count: Math.floor(diff / hour) })
  if (diff < day * 30) return t('project.relativeTime.daysAgo', { count: Math.floor(diff / day) })
  return new Intl.DateTimeFormat(locale, {
    month: '2-digit',
    day: '2-digit',
  }).format(timestamp)
}

/* ── 生成唯一 ID ── */
export function genId(prefix = 'p'): string {
  return `${prefix}-${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 7)}`
}

/* ── 将字符串数组转为 Tag 对象 ── */
export function toTagObjects(tags: string[]): ProjectTag[] {
  return tags.map((name, i) => ({ id: `tag-${i}-${name}`, name }))
}
