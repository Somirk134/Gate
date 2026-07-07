import type { Ref } from "vue"
import { computed } from "vue"
import type { LogItem } from "../types"
import { buildLogStatistics } from "../constants"

export function useLogStatistics(logs: Ref<LogItem[]>) {
  const statistics = computed(() => buildLogStatistics(logs.value))
  return { statistics }
}
