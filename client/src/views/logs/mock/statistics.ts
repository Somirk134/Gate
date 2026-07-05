import type { LogItem, LogStatistics } from "../types"

export function buildLogStatistics(logs: LogItem[]): LogStatistics {
  const start = new Date()
  start.setHours(0, 0, 0, 0)
  const todayStart = start.getTime()

  return {
    total: logs.length,
    error: logs.filter((log) => log.level === "ERROR").length,
    warning: logs.filter((log) => log.level === "WARN").length,
    info: logs.filter((log) => log.level === "INFO").length,
    today: logs.filter((log) => log.timestamp >= todayStart).length,
    fatal: logs.filter((log) => log.level === "FATAL").length,
    debug: logs.filter((log) => log.level === "DEBUG").length,
    trace: logs.filter((log) => log.level === "TRACE").length,
  }
}

export const emptyLogStatistics: LogStatistics = {
  total: 0,
  error: 0,
  warning: 0,
  info: 0,
  today: 0,
  fatal: 0,
  debug: 0,
  trace: 0,
}
