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
  { key: 'blue', label: '蓝色', value: '#5B8DEF' },
  { key: 'green', label: '绿色', value: '#22C55E' },
  { key: 'purple', label: '紫色', value: '#7C6FF2' },
  { key: 'orange', label: '橙色', value: '#F59E0B' },
  { key: 'red', label: '红色', value: '#EF4444' },
  { key: 'cyan', label: '青色', value: '#06B6D4' },
  { key: 'pink', label: '粉色', value: '#EC4899' },
  { key: 'indigo', label: '靛蓝', value: '#6366F1' },
  { key: 'teal', label: '蓝绿', value: '#14B8A6' },
  { key: 'amber', label: '琥珀', value: '#D97706' },
  { key: 'slate', label: '石板灰', value: '#64748B' },
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
  { key: 'package', label: '项目' },
  { key: 'globe', label: '域名' },
  { key: 'database', label: '数据库' },
  { key: 'servers', label: '服务器' },
  { key: 'cloud', label: '云服务' },
  { key: 'code', label: '代码' },
  { key: 'box', label: '容器' },
  { key: 'terminal', label: '终端' },
  { key: 'activity', label: '活动' },
  { key: 'cpu', label: 'CPU' },
  { key: 'router', label: '路由' },
  { key: 'layers', label: '分层' },
  { key: 'boxes', label: '资源组' },
  { key: 'network', label: '网络' },
  { key: 'shield', label: '安全' },
  { key: 'zap', label: '快速' },
  { key: 'rocket', label: '启动' },
  { key: 'hard-drive', label: '存储' },
  { key: 'link', label: '链接' },
  { key: 'plug', label: '插件' },
]

export const PROJECT_TEMPLATES: ProjectTemplateProfile[] = [
  {
    key: 'blank',
    label: '空白',
    icon: 'package',
    color: 'blue',
    description: '空白工作空间，自由组织资源。',
    tags: [],
    recommendations: [],
  },
  {
    key: 'springBoot',
    label: 'Spring Boot 服务',
    icon: 'code',
    color: 'green',
    description: 'Java Web 服务，推荐 HTTP 8080。',
    tags: ['后端', 'Java'],
    recommendations: [
      {
        id: 'spring-http',
        name: 'Spring Boot HTTP 服务',
        protocol: 'http',
        localHost: '127.0.0.1',
        localPort: 8080,
        remotePort: 18080,
        description: '暴露 Spring Boot 本地 HTTP 服务',
        tags: ['后端', 'HTTP'],
      },
    ],
  },
  {
    key: 'vue',
    label: 'Vue 前端',
    icon: 'code',
    color: 'cyan',
    description: '前端开发服务器，推荐 HTTP 5173。',
    tags: ['前端', 'Vite'],
    recommendations: [
      {
        id: 'vue-dev',
        name: 'Vue 开发服务',
        protocol: 'http',
        localHost: '127.0.0.1',
        localPort: 5173,
        remotePort: 15173,
        description: '暴露 Vite/Vue 开发服务器',
        tags: ['前端', 'HTTP'],
      },
    ],
  },
  {
    key: 'node',
    label: 'Node 服务',
    icon: 'terminal',
    color: 'green',
    description: 'Node.js API 或本地服务。',
    tags: ['Node', 'API'],
    recommendations: [
      {
        id: 'node-api',
        name: 'Node API 服务',
        protocol: 'http',
        localHost: '127.0.0.1',
        localPort: 3000,
        remotePort: 13000,
        description: '暴露 Node.js HTTP API',
        tags: ['Node', 'HTTP'],
      },
    ],
  },
  {
    key: 'python',
    label: 'Python 服务',
    icon: 'file-code',
    color: 'blue',
    description: 'Python Flask/FastAPI 服务。',
    tags: ['Python', 'API'],
    recommendations: [
      {
        id: 'python-api',
        name: 'Python API 服务',
        protocol: 'http',
        localHost: '127.0.0.1',
        localPort: 8000,
        remotePort: 18000,
        description: '暴露 Python Web 服务',
        tags: ['Python', 'HTTP'],
      },
    ],
  },
  {
    key: 'docker',
    label: 'Docker 容器',
    icon: 'box',
    color: 'indigo',
    description: '容器化服务入口。',
    tags: ['Docker'],
    recommendations: [
      {
        id: 'docker-web',
        name: 'Docker Web 服务',
        protocol: 'http',
        localHost: '127.0.0.1',
        localPort: 8080,
        remotePort: 18081,
        description: '暴露容器映射的 Web 服务',
        tags: ['Docker', 'HTTP'],
      },
    ],
  },
  {
    key: 'mcpServer',
    label: 'MCP 服务',
    icon: 'plug-zap',
    color: 'purple',
    description: '本地 MCP 工具服务。',
    tags: ['MCP', 'AI'],
    recommendations: [
      {
        id: 'mcp-server',
        name: 'MCP 服务',
        protocol: 'http',
        localHost: '127.0.0.1',
        localPort: 3333,
        remotePort: 13333,
        description: '暴露 MCP HTTP/SSE 服务',
        tags: ['MCP', 'AI'],
      },
    ],
  },
  {
    key: 'aiAgent',
    label: 'AI Agent',
    icon: 'sparkles',
    color: 'pink',
    description: 'Agent 调试、Webhook 或 Web UI。',
    tags: ['AI', 'Agent'],
    recommendations: [
      {
        id: 'agent-console',
        name: 'Agent 控制台',
        protocol: 'http',
        localHost: '127.0.0.1',
        localPort: 7860,
        remotePort: 17860,
        description: '暴露 Agent 控制台或调试端点',
        tags: ['AI', 'Agent'],
      },
    ],
  },
  {
    key: 'nas',
    label: 'NAS',
    icon: 'hard-drive',
    color: 'slate',
    description: '家庭 NAS、媒体与管理端口。',
    tags: ['家庭', '存储'],
    recommendations: [
      {
        id: 'nas-panel',
        name: 'NAS 管理面板',
        protocol: 'http',
        localHost: '127.0.0.1',
        localPort: 5000,
        remotePort: 15000,
        description: '暴露 NAS 管理面板',
        tags: ['NAS', '家庭'],
      },
    ],
  },
  {
    key: 'ssh',
    label: 'SSH',
    icon: 'terminal',
    color: 'amber',
    description: '远程 Shell 或设备维护。',
    tags: ['SSH'],
    recommendations: [
      {
        id: 'ssh',
        name: 'SSH',
        protocol: 'tcp',
        localHost: '127.0.0.1',
        localPort: 22,
        remotePort: 10022,
        description: '暴露 SSH TCP 端口',
        tags: ['SSH', 'TCP'],
      },
    ],
  },
  {
    key: 'git',
    label: 'Git',
    icon: 'git-branch',
    color: 'orange',
    description: 'Git 服务与代码托管。',
    tags: ['Git'],
    recommendations: [
      {
        id: 'git-http',
        name: 'Git HTTP',
        protocol: 'http',
        localHost: '127.0.0.1',
        localPort: 3000,
        remotePort: 13001,
        description: '暴露 Git Web 面板',
        tags: ['Git', 'HTTP'],
      },
    ],
  },
  {
    key: 'webhook',
    label: 'Webhook',
    icon: 'radio',
    color: 'teal',
    description: '本地 Webhook 回调调试。',
    tags: ['Webhook'],
    recommendations: [
      {
        id: 'webhook',
        name: 'Webhook 接收端',
        protocol: 'http',
        localHost: '127.0.0.1',
        localPort: 9000,
        remotePort: 19000,
        description: '暴露本地 Webhook 接收端',
        tags: ['Webhook', 'HTTP'],
      },
    ],
  },
  {
    key: 'custom',
    label: '自定义',
    icon: 'layers',
    color: 'blue',
    description: '自定义资源集合。',
    tags: ['自定义'],
    recommendations: [
      {
        id: 'custom-http',
        name: '自定义 HTTP 服务',
        protocol: 'http',
        localHost: '127.0.0.1',
        localPort: 8080,
        remotePort: 18082,
        description: '自定义 HTTP 服务',
        tags: ['自定义'],
      },
    ],
  },
]

/* ── 预置标签 ── */
export const PROJECT_TAGS: TagPreset[] = [
  { name: '工作', color: '#5B8DEF' },
  { name: '家庭', color: '#22C55E' },
  { name: '演示', color: '#F59E0B' },
  { name: '客户', color: '#EC4899' },
  { name: '开源', color: '#14B8A6' },
  { name: '个人', color: '#7C6FF2' },
  { name: '生产', color: '#EF4444' },
  { name: '预发', color: '#06B6D4' },
]

/* ── 状态配置：label + GBadge variant ── */
export const STATUS_CONFIG: Record<
  ProjectStatus,
  { label: string; variant: 'success' | 'warning' | 'error' | 'neutral' | 'info'; pulse: boolean }
> = {
  running: { label: '运行中', variant: 'success', pulse: false },
  partial: { label: '部分运行', variant: 'warning', pulse: false },
  stopped: { label: '已停止', variant: 'neutral', pulse: false },
  starting: { label: '启动中', variant: 'info', pulse: true },
  error: { label: '异常', variant: 'error', pulse: false },
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

export function formatRelativeTime(timestamp: number): string {
  if (!timestamp) return '从未'
  const diff = Date.now() - timestamp
  const minute = 60 * 1000
  const hour = 60 * minute
  const day = 24 * hour
  if (diff < minute) return '刚刚'
  if (diff < hour) return `${Math.floor(diff / minute)} 分钟前`
  if (diff < day) return `${Math.floor(diff / hour)} 小时前`
  if (diff < day * 30) return `${Math.floor(diff / day)} 天前`
  return new Intl.DateTimeFormat('zh-CN', {
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
