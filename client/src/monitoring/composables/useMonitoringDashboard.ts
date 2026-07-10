import { computed, onMounted, onUnmounted, readonly, shallowRef, ref } from 'vue'
import { createEmptyDashboardData, dashboardService } from '../services'
import type { DashboardData, RealtimeSpeedPoint } from '../types'

export interface RuntimeMetricHistoryPoint {
  timestamp: number
  uploadBps: number
  downloadBps: number
  connection: number
  requests: number
  errors: number
  latencyMs: number
  cpuUsage: number
  memoryUsage: number
  reconnect: number
}

export function useMonitoringDashboard() {
  const data = shallowRef<DashboardData>(createEmptyDashboardData())
  const loading = ref(false)
  const error = ref<string | null>(null)
  let realtimeSpeedHistory: RealtimeSpeedPoint[] = []
  const metricHistory = ref<RuntimeMetricHistoryPoint[]>([])
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
    metricHistory: readonly(metricHistory),
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
    appendMetricHistory(next)

    return {
      ...next,
      realtimeSpeed: realtimeSpeedHistory,
    }
  }

  function appendMetricHistory(next: DashboardData) {
    const statistics = next.statistics
    const http = statistics.http
    const timestamp = normalizeMetricTimestamp(next.generatedAt || statistics.collectedAt)
    const latest = metricHistory.value[metricHistory.value.length - 1]
    const point: RuntimeMetricHistoryPoint = {
      timestamp,
      uploadBps: normalizeMetricValue(statistics.traffic.uploadSpeedBps),
      downloadBps: normalizeMetricValue(statistics.traffic.downloadSpeedBps),
      connection: normalizeMetricValue(statistics.connection.currentConnection),
      requests: normalizeMetricValue(http?.requestsTotal),
      errors: normalizeMetricValue(
        Object.entries(http?.statusCodes ?? {}).reduce(
          (sum, [status, count]) => sum + (Number(status) >= 400 ? Number(count) : 0),
          0,
        ),
      ),
      latencyMs: normalizeMetricValue(statistics.connection.averageRttMs),
      cpuUsage: normalizeMetricValue(statistics.system.cpuUsage),
      memoryUsage: normalizeMetricValue(statistics.system.memoryUsage),
      reconnect: normalizeMetricValue(statistics.connection.reconnect),
    }

    // 只保存真实 Runtime 快照的短历史，供图表平滑增量更新使用。
    if (latest?.timestamp === timestamp) {
      metricHistory.value = [...metricHistory.value.slice(0, -1), point]
      return
    }

    metricHistory.value = [...metricHistory.value, point].slice(-120)
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

  function normalizeMetricTimestamp(timestamp: number) {
    return Number.isFinite(timestamp) ? timestamp : Date.now()
  }

  function normalizeMetricValue(value: number | undefined) {
    return Number.isFinite(value) ? Math.max(0, Number(value)) : 0
  }
}
