import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { i18n } from '@/i18n'
import { TauriIpcClient } from '@/ipc'
import { buildLogStatistics } from '../constants'
import type { LogFilter, LogItem, LogLevel, LogLoadStatus, LogSource } from '../types'
import { getTimeRangeStart, normalizeText } from '../utils'

interface RuntimeLogRecord {
  level: string
  source: string
  module?: string
  message: string
  timestamp: number
  tunnelId?: string
  tunnel_id?: string
  serverId?: string
  host?: string
  method?: string
  path?: string
  status?: number
  statusCode?: number
  latencyMs?: number
  bytes?: number
  bytesIn?: number
  bytesOut?: number
  scheme?: string
  tlsVersion?: string | null
  sni?: string | null
}

const ipc = new TauriIpcClient()

function t(key: string, params?: Record<string, unknown>): string {
  return (i18n.global as unknown as { t: (key: string, params?: Record<string, unknown>) => string }).t(
    key,
    params,
  )
}

export const defaultLogFilter: LogFilter = {
  levels: [],
  sources: [],
  modules: [],
  projects: [],
  tunnels: [],
  timeRange: 'all',
  keyword: '',
  groupBy: 'none',
  fuzzy: true,
}

function matchKeyword(log: LogItem, keyword: string, fuzzy: boolean): boolean {
  const query = normalizeText(keyword)
  if (!query) return true

  const haystack = normalizeText(
    [
      log.level,
      log.source,
      log.module,
      log.message,
      log.projectName,
      log.tunnelName,
      log.traceId,
      log.requestId,
      log.raw,
    ]
      .filter(Boolean)
      .join(' '),
  )

  if (haystack.includes(query)) return true
  if (!fuzzy) return false

  return haystack.split(/[\s/._:-]+/).some((token) => {
    let cursor = 0
    for (const char of query) {
      cursor = token.indexOf(char, cursor)
      if (cursor === -1) return false
      cursor += 1
    }
    return true
  })
}

export const useLogStore = defineStore('log-center', () => {
  const logs = ref<LogItem[]>([])
  const status = ref<LogLoadStatus>('idle')
  const error = ref('')
  const filter = ref<LogFilter>({ ...defaultLogFilter })
  const paused = ref(false)
  const autoScroll = ref(true)
  const selectedId = ref<string | null>(null)
  const lastUpdated = ref(0)
  const droppedCount = ref(0)
  const maxRetained = ref(100_000)

  const isLoading = computed(() => status.value === 'loading')
  const isError = computed(() => status.value === 'error')
  const isReady = computed(() => status.value === 'success')
  const hasLogs = computed(() => logs.value.length > 0)
  const statistics = computed(() => buildLogStatistics(logs.value))

  const availableModules = computed(() =>
    Array.from(new Set(logs.value.map((log) => log.module))).sort(),
  )
  const availableProjects = computed(() =>
    Array.from(
      new Set(logs.value.map((log) => log.projectName).filter(Boolean) as string[]),
    ).sort(),
  )
  const availableTunnels = computed(() =>
    Array.from(new Set(logs.value.map((log) => log.tunnelName).filter(Boolean) as string[])).sort(),
  )

  const filteredLogs = computed(() => filterLogs(logs.value, filter.value))
  const selectedLog = computed(() =>
    selectedId.value ? (logs.value.find((log) => log.id === selectedId.value) ?? null) : null,
  )
  const filteredStatistics = computed(() => buildLogStatistics(filteredLogs.value))

  function filterLogs(input: LogItem[], current: LogFilter): LogItem[] {
    const start = getTimeRangeStart(current.timeRange)
    return input
      .filter((log) => {
        if (current.levels.length && !current.levels.includes(log.level)) return false
        if (current.sources.length && !current.sources.includes(log.source)) return false
        if (current.modules.length && !current.modules.includes(log.module)) return false
        if (current.projects.length && !log.projectName) return false
        if (current.projects.length && !current.projects.includes(log.projectName ?? '')) return false
        if (current.tunnels.length && !log.tunnelName) return false
        if (current.tunnels.length && !current.tunnels.includes(log.tunnelName ?? '')) return false
        if (start && log.timestamp < start) return false
        return matchKeyword(log, current.keyword, current.fuzzy)
      })
      .sort((left, right) => right.timestamp - left.timestamp)
  }

  async function load(): Promise<void> {
    status.value = 'loading'
    error.value = ''
    try {
      const runtimeLogs = await ipc.invoke<RuntimeLogRecord[]>('runtime_get_logs')
      logs.value = runtimeLogs.map(mapRuntimeLog).sort((left, right) => left.timestamp - right.timestamp)
      status.value = 'success'
      lastUpdated.value = Date.now()
      if (!selectedId.value && logs.value.length)
        selectedId.value = logs.value[logs.value.length - 1].id
    } catch (e) {
      status.value = 'error'
      error.value = e instanceof Error ? e.message : t('logs.loadingFailed')
    }
  }

  async function refresh(): Promise<void> {
    return load()
  }

  function append(log: LogItem): void {
    if (paused.value) return
    logs.value.push(log)
    trimRetained()
    lastUpdated.value = Date.now()
  }

  function appendMany(items: LogItem[]): void {
    if (paused.value) return
    logs.value.push(...items)
    trimRetained()
    lastUpdated.value = Date.now()
  }

  function remove(id: string): void {
    const index = logs.value.findIndex((log) => log.id === id)
    if (index === -1) return
    logs.value.splice(index, 1)
    if (selectedId.value === id) selectedId.value = null
  }

  function clear(): void {
    logs.value = []
    selectedId.value = null
    droppedCount.value = 0
    lastUpdated.value = Date.now()
  }

  function setFilter(patch: Partial<LogFilter>): void {
    filter.value = { ...filter.value, ...patch }
  }

  function resetFilter(): void {
    filter.value = { ...defaultLogFilter }
  }

  function search(keyword: string): void {
    setFilter({ keyword })
  }

  function select(id: string | null): void {
    selectedId.value = id
  }

  function pause(): void {
    paused.value = true
    autoScroll.value = false
  }

  function resume(): void {
    paused.value = false
    autoScroll.value = true
  }

  function setAutoScroll(value: boolean): void {
    autoScroll.value = value
    if (value) paused.value = false
  }

  function trimRetained(): void {
    const overflow = logs.value.length - maxRetained.value
    if (overflow <= 0) return
    logs.value.splice(0, overflow)
    droppedCount.value += overflow
  }

  function setLevel(level: LogLevel, enabled: boolean): void {
    const next = new Set(filter.value.levels)
    if (enabled) next.add(level)
    else next.delete(level)
    setFilter({ levels: Array.from(next) })
  }

  function setSource(source: LogSource | 'ALL'): void {
    setFilter({ sources: source === 'ALL' ? [] : [source] })
  }

  return {
    logs,
    status,
    error,
    filter,
    paused,
    autoScroll,
    selectedId,
    lastUpdated,
    droppedCount,
    maxRetained,
    isLoading,
    isError,
    isReady,
    hasLogs,
    statistics,
    filteredStatistics,
    filteredLogs,
    selectedLog,
    availableModules,
    availableProjects,
    availableTunnels,
    load,
    refresh,
    append,
    appendMany,
    remove,
    clear,
    filterLogs,
    setFilter,
    resetFilter,
    search,
    select,
    pause,
    resume,
    setAutoScroll,
    setLevel,
    setSource,
  }
})

function mapRuntimeLog(log: RuntimeLogRecord): LogItem {
  const level = normalizeLevel(log.level)
  const source = normalizeSource(log.source)
  const raw = JSON.stringify(log)
  const tunnelId = log.tunnelId ?? log.tunnel_id
  const statusCode = normalizeOptionalNumber(log.statusCode ?? log.status)
  const bytesOut = normalizeOptionalNumber(log.bytesOut ?? log.bytes)

  return {
    id: `${log.source}-${log.timestamp}-${log.message}`,
    timestamp: log.timestamp,
    level,
    source,
    module: log.module || log.source || 'runtime',
    message: log.message,
    tunnelId,
    tunnelName: tunnelId,
    context: {
      environment: source === 'SERVER' ? 'server' : 'desktop',
      host: log.host || 'local',
      processId: 0,
      thread: log.module || log.source || 'runtime',
      sessionId: log.serverId || '',
    },
    metadata: {
      durationMs: normalizeOptionalNumber(log.latencyMs),
      statusCode,
      method: log.method,
      path: log.path,
      bytesIn: normalizeOptionalNumber(log.bytesIn),
      bytesOut,
      tags: [
        log.source,
        tunnelId,
        log.serverId,
        log.method,
        statusCode ? `status:${statusCode}` : '',
        log.scheme,
      ].filter(Boolean) as string[],
    },
    raw,
  }
}

function normalizeOptionalNumber(value: unknown): number | undefined {
  return typeof value === 'number' && Number.isFinite(value) ? value : undefined
}

function normalizeLevel(level: string): LogLevel {
  const value = level.toUpperCase()
  if (
    value === 'TRACE' ||
    value === 'DEBUG' ||
    value === 'INFO' ||
    value === 'WARN' ||
    value === 'ERROR' ||
    value === 'FATAL'
  ) {
    return value
  }
  return 'INFO'
}

function normalizeSource(source: string): LogSource {
  const value = source.toLowerCase()
  if (value.includes('tunnel')) return 'TUNNEL'
  if (value.includes('server')) return 'SERVER'
  if (value.includes('stat') || value.includes('metric')) return 'STATISTICS'
  if (value.includes('update')) return 'UPDATE'
  if (value.includes('project')) return 'PROJECT'
  if (
    value.includes('client') ||
    value.includes('connection') ||
    value.includes('auth') ||
    value.includes('heartbeat')
  )
    return 'CLIENT'
  return 'SYSTEM'
}
