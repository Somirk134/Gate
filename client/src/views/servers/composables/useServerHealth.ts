/* ==================================================================
   useServerHealth — 服务器健康检查组合式函数
   ------------------------------------------------------------------
   封装健康检查执行、轮询、状态派生。
   全部 Mock，未来替换为真实健康检查 API。
   ================================================================== */

import { computed, ref, type Ref } from "vue"
import type { Server, ServerHealthItem } from "../types"

export function useServerHealth(server: Ref<Server | undefined>) {
  const checking = ref(false)
  const lastCheckedAt = ref<number>(0)

  const health = computed(() => server.value?.health)
  const overall = computed(() => health.value?.overall ?? "unknown")
  const score = computed(() => health.value?.score ?? 0)
  const items = computed<ServerHealthItem[]>(() => health.value?.items ?? [])

  const passCount = computed(() => items.value.filter((i) => i.status === "pass").length)
  const warnCount = computed(() => items.value.filter((i) => i.status === "warn").length)
  const failCount = computed(() => items.value.filter((i) => i.status === "fail").length)

  const isHealthy = computed(() => overall.value === "healthy")
  const hasWarning = computed(() => overall.value === "warning")
  const isCritical = computed(() => overall.value === "critical")

  /** 执行检查（实际由 store.checkHealth 完成） */
  function runCheck(checkFn: () => void) {
    if (checking.value) return
    checking.value = true
    checkFn()
    lastCheckedAt.value = Date.now()
    setTimeout(() => {
      checking.value = false
    }, 1100)
  }

  return {
    checking,
    lastCheckedAt,
    health,
    overall,
    score,
    items,
    passCount,
    warnCount,
    failCount,
    isHealthy,
    hasWarning,
    isCritical,
    runCheck,
  }
}
