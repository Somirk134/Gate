import { computed, onMounted, onUnmounted, shallowRef, ref } from 'vue'
import { createEmptyDashboardData, dashboardService } from '../services'
import type { DashboardData, RealtimeSpeedPoint } from '../types'

export function useMonitoringDashboard() {
  const data = shallowRef<DashboardData>(createEmptyDashboardData())
  const loading = ref(false)
  const error = ref<string | null>(null)
  let realtimeSpeedHistory: RealtimeSpeedPoint[] = []
  let unsubscribe: (() => void) | undefined

  const dashboard = computed(() => data.value)
  const healthStatus = computed(() => data.value.systemHealth.overall)
  const lastUpdated = computed(() => new Date(data.value.generatedAt))

  async function refresh() {
    loading.value = true
    error.value = null
    try {
      data.value = withRealtimeSpeedHistory(await dashboardService.getDashboard())
    } catch (err) {
      error.value = err instanceof Error ? err.message : 'Failed to refresh monitoring data'
    } finally {
      loading.value = false
    }
  }

  onMounted(async () => {
    await refresh()
    unsubscribe = dashboardService.subscribe((next) => {
      data.value = withRealtimeSpeedHistory(next)
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

  function withRealtimeSpeedHistory(next: DashboardData): DashboardData {
    const points = next.realtimeSpeed.length > 0 ? next.realtimeSpeed : [createCurrentSpeedPoint(next)]

    // 后端推送的是当前真实采样点，前端保留最近 26 个点用于稳定展示趋势。
    for (const point of points) {
      const normalized = normalizeSpeedPoint(point)
      const latest = realtimeSpeedHistory[realtimeSpeedHistory.length - 1]

      if (latest?.timestamp === normalized.timestamp) {
        realtimeSpeedHistory[realtimeSpeedHistory.length - 1] = normalized
      } else {
        realtimeSpeedHistory.push(normalized)
      }
    }

    realtimeSpeedHistory = realtimeSpeedHistory.slice(-26)

    return {
      ...next,
      realtimeSpeed: realtimeSpeedHistory,
    }
  }

  function createCurrentSpeedPoint(next: DashboardData): RealtimeSpeedPoint {
    return {
      timestamp: next.generatedAt || Date.now(),
      uploadBps: next.statistics.traffic.uploadSpeedBps,
      downloadBps: next.statistics.traffic.downloadSpeedBps,
    }
  }

  function normalizeSpeedPoint(point: RealtimeSpeedPoint): RealtimeSpeedPoint {
    return {
      timestamp: Number.isFinite(point.timestamp) ? point.timestamp : Date.now(),
      uploadBps: Number.isFinite(point.uploadBps) ? Math.max(0, point.uploadBps) : 0,
      downloadBps: Number.isFinite(point.downloadBps) ? Math.max(0, point.downloadBps) : 0,
    }
  }
}
