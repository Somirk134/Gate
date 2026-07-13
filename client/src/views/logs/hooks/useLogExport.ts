import type { Ref } from 'vue'
import type { LogItem } from '../types'
import { downloadLogs, serializeLogs, type LogExportFormat, type LogExportResult } from '../utils'

export function useLogExport(logs: Ref<LogItem[]>) {
  function exportLogs(format: LogExportFormat): LogExportResult {
    return downloadLogs(logs.value, format)
  }

  async function copyLogs(format: LogExportFormat): Promise<void> {
    await navigator.clipboard.writeText(serializeLogs(logs.value, format))
  }

  return {
    exportLogs,
    copyLogs,
  }
}
