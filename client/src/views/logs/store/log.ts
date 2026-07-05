import { defineStore } from "pinia"
import { computed, ref } from "vue"
import type { LogFilter, LogItem, LogLevel, LogLoadStatus, LogSource } from "../types"
import { buildLogStatistics, createMockLog, mockLogs } from "../mock"
import { getTimeRangeStart, normalizeText } from "../utils"

export const defaultLogFilter: LogFilter = {
  levels: [],
  sources: [],
  modules: [],
  projects: [],
  tunnels: [],
  timeRange: "all",
  keyword: "",
  groupBy: "none",
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
      .join(" "),
  )

  if (haystack.includes(query)) return true
  if (!fuzzy) return false

  return haystack
    .split(/[\s/._:-]+/)
    .some((token) => {
      let cursor = 0
      for (const char of query) {
        cursor = token.indexOf(char, cursor)
        if (cursor === -1) return false
        cursor += 1
      }
      return true
    })
}

export const useLogStore = defineStore("log-center", () => {
  const logs = ref<LogItem[]>([])
  const status = ref<LogLoadStatus>("idle")
  const error = ref("")
  const filter = ref<LogFilter>({ ...defaultLogFilter })
  const paused = ref(false)
  const autoScroll = ref(true)
  const selectedId = ref<string | null>(null)
  const lastUpdated = ref(0)
  const droppedCount = ref(0)
  const maxRetained = ref(100_000)

  const isLoading = computed(() => status.value === "loading")
  const isError = computed(() => status.value === "error")
  const isReady = computed(() => status.value === "success")
  const hasLogs = computed(() => logs.value.length > 0)
  const statistics = computed(() => buildLogStatistics(logs.value))

  const availableModules = computed(() =>
    Array.from(new Set(logs.value.map((log) => log.module))).sort(),
  )
  const availableProjects = computed(() =>
    Array.from(new Set(logs.value.map((log) => log.projectName).filter(Boolean) as string[])).sort(),
  )
  const availableTunnels = computed(() =>
    Array.from(new Set(logs.value.map((log) => log.tunnelName).filter(Boolean) as string[])).sort(),
  )

  const filteredLogs = computed(() => filterLogs(logs.value, filter.value))
  const selectedLog = computed(() =>
    selectedId.value ? logs.value.find((log) => log.id === selectedId.value) ?? null : null,
  )
  const filteredStatistics = computed(() => buildLogStatistics(filteredLogs.value))

  function filterLogs(input: LogItem[], current: LogFilter): LogItem[] {
    const start = getTimeRangeStart(current.timeRange)
    return input.filter((log) => {
      if (current.levels.length && !current.levels.includes(log.level)) return false
      if (current.sources.length && !current.sources.includes(log.source)) return false
      if (current.modules.length && !current.modules.includes(log.module)) return false
      if (current.projects.length && !log.projectName) return false
      if (current.projects.length && !current.projects.includes(log.projectName ?? "")) return false
      if (current.tunnels.length && !log.tunnelName) return false
      if (current.tunnels.length && !current.tunnels.includes(log.tunnelName ?? "")) return false
      if (start && log.timestamp < start) return false
      return matchKeyword(log, current.keyword, current.fuzzy)
    })
  }

  async function load(): Promise<void> {
    status.value = "loading"
    error.value = ""
    try {
      await new Promise((resolve) => setTimeout(resolve, 350))
      logs.value = structuredClone(mockLogs)
      status.value = "success"
      lastUpdated.value = Date.now()
      if (!selectedId.value && logs.value.length) selectedId.value = logs.value[logs.value.length - 1].id
    } catch (e) {
      status.value = "error"
      error.value = e instanceof Error ? e.message : "Failed to load logs"
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

  function generateTestLogs(count = 1000): void {
    logs.value = Array.from({ length: count }, (_, index) =>
      createMockLog(index + 1, Date.now() - (count - index) * 15_000),
    )
    status.value = "success"
    selectedId.value = logs.value[logs.value.length - 1]?.id ?? null
    lastUpdated.value = Date.now()
  }

  function appendTestLog(): void {
    append(createMockLog(logs.value.length + 1, Date.now()))
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

  function setSource(source: LogSource | "ALL"): void {
    setFilter({ sources: source === "ALL" ? [] : [source] })
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
    generateTestLogs,
    appendTestLog,
    setLevel,
    setSource,
  }
})
