import type { Ref } from "vue"
import { computed } from "vue"
import type { LogItem } from "../types"
import { normalizeText } from "../utils"

export function useLogSearch(logs: Ref<LogItem[]>, keyword: Ref<string>) {
  const normalizedKeyword = computed(() => normalizeText(keyword.value))
  const matched = computed(() => {
    if (!normalizedKeyword.value) return logs.value
    return logs.value.filter((log) =>
      normalizeText(`${log.message} ${log.module} ${log.source} ${log.traceId ?? ""}`).includes(
        normalizedKeyword.value,
      ),
    )
  })

  return {
    normalizedKeyword,
    matched,
  }
}
