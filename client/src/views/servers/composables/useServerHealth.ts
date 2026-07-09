import { computed, ref, type Ref } from 'vue'
import type { Server, ServerHealthItem } from '../types'

export function useServerHealth(server: Ref<Server | undefined>) {
  const checking = ref(false)
  const lastCheckedAt = ref<number>(0)

  const health = computed(() => server.value?.health)
  const overall = computed(() => health.value?.overall ?? 'unknown')
  const score = computed(() => health.value?.score ?? 0)
  const items = computed<ServerHealthItem[]>(() => health.value?.items ?? [])

  const passCount = computed(() => items.value.filter((item) => item.status === 'pass').length)
  const warnCount = computed(() => items.value.filter((item) => item.status === 'warn').length)
  const failCount = computed(() => items.value.filter((item) => item.status === 'fail').length)

  const isHealthy = computed(() => overall.value === 'healthy')
  const hasWarning = computed(() => overall.value === 'warning')
  const isCritical = computed(() => overall.value === 'critical')

  function runCheck(checkFn: () => void) {
    if (checking.value) return
    checking.value = true
    try {
      checkFn()
      lastCheckedAt.value = Date.now()
    } finally {
      checking.value = false
    }
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
