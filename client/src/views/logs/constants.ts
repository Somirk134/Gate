import type {
  LogItem,
  LogLevel,
  LogLevelOption,
  LogSource,
  LogSourceNode,
  LogStatistics,
} from './types'

export const LOG_LEVELS: LogLevelOption[] = [
  {
    level: 'TRACE',
    label: '跟踪',
    color: '#8A9099',
    muted: 'rgba(138, 144, 153, 0.12)',
    icon: 'circle-dot',
  },
  {
    level: 'DEBUG',
    label: '调试',
    color: '#06B6D4',
    muted: 'rgba(6, 182, 212, 0.12)',
    icon: 'code',
  },
  {
    level: 'INFO',
    label: '信息',
    color: '#3B82F6',
    muted: 'rgba(59, 130, 246, 0.12)',
    icon: 'info-circle',
  },
  {
    level: 'WARN',
    label: '警告',
    color: '#F59E0B',
    muted: 'rgba(245, 158, 11, 0.14)',
    icon: 'alert-triangle',
  },
  {
    level: 'ERROR',
    label: '错误',
    color: '#EF4444',
    muted: 'rgba(239, 68, 68, 0.13)',
    icon: 'alert-circle',
  },
  {
    level: 'FATAL',
    label: '严重',
    color: '#A855F7',
    muted: 'rgba(168, 85, 247, 0.14)',
    icon: 'zap',
  },
]

export const LOG_SOURCE_LIST: LogSource[] = [
  'SYSTEM',
  'CLIENT',
  'SERVER',
  'PROJECT',
  'TUNNEL',
  'STATISTICS',
  'UPDATE',
  'PLUGIN',
]

export const LOG_SOURCE_LABELS: Record<LogSource, string> = {
  SYSTEM: '系统',
  CLIENT: '客户端',
  SERVER: '服务器',
  PROJECT: '项目',
  TUNNEL: '隧道',
  STATISTICS: '统计',
  UPDATE: '更新',
  PLUGIN: '插件',
}

export const LOG_SOURCES: LogSourceNode[] = [
  {
    id: 'ALL',
    label: '全部日志',
    icon: 'logs',
    children: LOG_SOURCE_LIST.map((source) => ({
      id: source,
      label: LOG_SOURCE_LABELS[source],
      icon: sourceIcon(source),
      reserved: source === 'PLUGIN',
    })),
  },
]

export function getLevelOption(level: LogLevel): LogLevelOption {
  return LOG_LEVELS.find((item) => item.level === level) ?? LOG_LEVELS[2]
}

export function buildLogStatistics(logs: LogItem[]): LogStatistics {
  const start = new Date()
  start.setHours(0, 0, 0, 0)
  const todayStart = start.getTime()

  return {
    total: logs.length,
    error: logs.filter((log) => log.level === 'ERROR').length,
    warning: logs.filter((log) => log.level === 'WARN').length,
    info: logs.filter((log) => log.level === 'INFO').length,
    today: logs.filter((log) => log.timestamp >= todayStart).length,
    fatal: logs.filter((log) => log.level === 'FATAL').length,
    debug: logs.filter((log) => log.level === 'DEBUG').length,
    trace: logs.filter((log) => log.level === 'TRACE').length,
  }
}

function sourceIcon(source: LogSource) {
  const icons: Record<LogSource, string> = {
    SYSTEM: 'monitor',
    CLIENT: 'terminal',
    SERVER: 'servers',
    PROJECT: 'projects',
    TUNNEL: 'router',
    STATISTICS: 'chart-bar',
    UPDATE: 'refresh',
    PLUGIN: 'plug',
  }
  return icons[source]
}
