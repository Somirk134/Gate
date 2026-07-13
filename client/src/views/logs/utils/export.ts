import type { LogItem } from '../types'

export type LogExportFormat = 'json' | 'txt' | 'ndjson'

export interface LogExportResult {
  filename: string
}

export function serializeLogs(logs: LogItem[], format: LogExportFormat): string {
  if (format === 'json') return JSON.stringify(logs, null, 2)
  if (format === 'ndjson') return logs.map((log) => JSON.stringify(log)).join('\n')
  return logs
    .map((log) => {
      const time = new Date(log.timestamp).toISOString()
      return `[${time}] ${log.level.padEnd(5)} ${log.source.padEnd(10)} ${log.module} ${log.message}`
    })
    .join('\n')
}

export function downloadLogs(logs: LogItem[], format: LogExportFormat): LogExportResult {
  const content = serializeLogs(logs, format)
  const type = format === 'json' || format === 'ndjson' ? 'application/json' : 'text/plain'
  const blob = new Blob([content], { type: `${type};charset=utf-8` })
  const url = URL.createObjectURL(blob)
  const filename = `gate-logs-${new Date().toISOString().replace(/[:.]/g, '-')}.${format}`
  const link = document.createElement('a')
  link.href = url
  link.download = filename
  // 浏览器下载 API 不返回真实保存路径；这里至少稳定返回文件名用于提示用户。
  document.body.appendChild(link)
  link.click()
  link.remove()
  window.setTimeout(() => URL.revokeObjectURL(url), 0)
  return { filename }
}
