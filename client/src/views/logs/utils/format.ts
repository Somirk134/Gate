import type { LogTimeRange } from "../types"

export function formatLogTime(timestamp: number): string {
  const date = new Date(timestamp)
  return `${String(date.getHours()).padStart(2, "0")}:${String(date.getMinutes()).padStart(2, "0")}:${String(date.getSeconds()).padStart(2, "0")}.${String(date.getMilliseconds()).padStart(3, "0")}`
}

export function formatLogDate(timestamp: number): string {
  const date = new Date(timestamp)
  return `${date.getFullYear()}-${String(date.getMonth() + 1).padStart(2, "0")}-${String(date.getDate()).padStart(2, "0")}`
}

export function formatBytes(bytes = 0): string {
  if (bytes < 1024) return `${bytes} B`
  const units = ["KB", "MB", "GB", "TB"]
  let value = bytes / 1024
  let unit = 0
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024
    unit += 1
  }
  return `${value.toFixed(value >= 100 ? 0 : 1)} ${units[unit]}`
}

export function getTimeRangeStart(range: LogTimeRange): number {
  const now = Date.now()
  if (range === "15m") return now - 15 * 60_000
  if (range === "1h") return now - 60 * 60_000
  if (range === "24h") return now - 24 * 60 * 60_000
  if (range === "today") {
    const start = new Date()
    start.setHours(0, 0, 0, 0)
    return start.getTime()
  }
  return 0
}

export function normalizeText(value: string): string {
  return value.trim().toLowerCase()
}
