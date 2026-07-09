import type { Ref } from 'vue'
import { computed } from 'vue'
import type { LogFilter, LogItem } from '../types'
import { getTimeRangeStart } from '../utils'

export function useLogFilter(logs: Ref<LogItem[]>, filter: Ref<LogFilter>) {
  const results = computed(() => {
    const start = getTimeRangeStart(filter.value.timeRange)
    return logs.value.filter((log) => {
      if (filter.value.levels.length && !filter.value.levels.includes(log.level)) return false
      if (filter.value.sources.length && !filter.value.sources.includes(log.source)) return false
      if (filter.value.modules.length && !filter.value.modules.includes(log.module)) return false
      if (start && log.timestamp < start) return false
      return true
    })
  })

  return { results }
}
