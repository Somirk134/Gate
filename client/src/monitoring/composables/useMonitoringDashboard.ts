import { computed, onMounted, onUnmounted, shallowRef, ref } from "vue"
import { createEmptyDashboardData, dashboardService } from "../services"
import type { DashboardData } from "../types"

export function useMonitoringDashboard() {
  const data = shallowRef<DashboardData>(createEmptyDashboardData())
  const loading = ref(false)
  const error = ref<string | null>(null)
  let unsubscribe: (() => void) | undefined

  const dashboard = computed(() => data.value)
  const healthStatus = computed(() => data.value.systemHealth.overall)
  const lastUpdated = computed(() => new Date(data.value.generatedAt))

  async function refresh() {
    loading.value = true
    error.value = null
    try {
      data.value = await dashboardService.getDashboard()
    } catch (err) {
      error.value = err instanceof Error ? err.message : "Failed to refresh monitoring data"
    } finally {
      loading.value = false
    }
  }

  onMounted(async () => {
    await refresh()
    unsubscribe = dashboardService.subscribe((next) => {
      data.value = next
    })
  })

  onUnmounted(() => {
    unsubscribe?.()
  })

  return {
    dashboard,
    healthStatus,
    lastUpdated,
    loading,
    error,
    refresh,
  }
}
