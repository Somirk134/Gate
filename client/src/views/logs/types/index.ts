export type LogLevel = "TRACE" | "DEBUG" | "INFO" | "WARN" | "ERROR" | "FATAL"

export type LogSource =
  | "SYSTEM"
  | "CLIENT"
  | "SERVER"
  | "PROJECT"
  | "TUNNEL"
  | "STATISTICS"
  | "UPDATE"
  | "PLUGIN"

export type LogTimeRange = "all" | "15m" | "1h" | "24h" | "today"

export type LogGroupBy = "none" | "time" | "source" | "level"

export interface LogContext {
  environment: "desktop" | "server" | "client"
  host: string
  processId: number
  thread: string
  sessionId: string
}

export interface LogMetadata {
  durationMs?: number
  statusCode?: number
  method?: string
  path?: string
  ip?: string
  userAgent?: string
  bytesIn?: number
  bytesOut?: number
  retry?: number
  tags: string[]
}

export interface LogItem {
  id: string
  timestamp: number
  level: LogLevel
  source: LogSource
  module: string
  message: string
  projectId?: string
  projectName?: string
  tunnelId?: string
  tunnelName?: string
  traceId?: string
  requestId?: string
  context: LogContext
  metadata: LogMetadata
  stack?: string[]
  raw: string
}

export interface LogFilter {
  levels: LogLevel[]
  sources: LogSource[]
  modules: string[]
  projects: string[]
  tunnels: string[]
  timeRange: LogTimeRange
  keyword: string
  groupBy: LogGroupBy
  fuzzy: boolean
}

export interface LogStatistics {
  total: number
  error: number
  warning: number
  info: number
  today: number
  fatal: number
  debug: number
  trace: number
}

export interface LogSourceNode {
  id: LogSource | "ALL"
  label: string
  icon: string
  children?: LogSourceNode[]
  reserved?: boolean
}

export interface LogLevelOption {
  level: LogLevel
  label: string
  color: string
  muted: string
  icon: string
}

export type LogLoadStatus = "idle" | "loading" | "success" | "error"
