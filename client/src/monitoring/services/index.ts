import { GateAppError, TauriIpcClient } from '@/ipc'
import { isTauri } from '@tauri-apps/api/core'
import type { DashboardData, HealthReport, Metric, Statistics, TrafficStatistics } from '../types'

const ipc = new TauriIpcClient()

/** 客户端统计服务契约。 */
export interface StatisticsService {
  getStatistics(): Promise<Statistics>
  getTraffic(): Promise<TrafficStatistics>
}

/** 客户端指标服务契约。 */
export interface MetricsService {
  collect(): Promise<Metric[]>
}

/** 客户端健康检查服务契约。 */
export interface HealthService {
  getHealthReport(): Promise<HealthReport>
}

/** 客户端首页监控服务契约。 */
export interface DashboardService {
  getDashboard(): Promise<DashboardData>
  subscribe(listener: (data: DashboardData) => void, onError?: (error: unknown) => void): () => void
}

/** 客户端导出服务契约。 */
export interface ExportService {
  exportJson(): Promise<string>
  exportCsv(): Promise<string>
}

export function createEmptyDashboardData(now = Date.now()): DashboardData {
  const traffic: TrafficStatistics = {
    uploadBytes: 0,
    downloadBytes: 0,
    uploadSpeedBps: 0,
    downloadSpeedBps: 0,
    peakSpeedBps: 0,
    averageSpeedBps: 0,
    todayTrafficBytes: 0,
    totalTrafficBytes: 0,
  }
  const statistics: Statistics = {
    collectedAt: now,
    tunnel: {
      tunnelCount: 0,
      runningTunnel: 0,
      stoppedTunnel: 0,
      upload: 0,
      download: 0,
      peakSpeedBps: 0,
      averageSpeedBps: 0,
      runningTimeSeconds: 0,
      todayTraffic: 0,
      totalTraffic: 0,
    },
    traffic,
    connection: {
      currentConnection: 0,
      totalConnection: 0,
      success: 0,
      failure: 0,
      reconnect: 0,
      disconnect: 0,
      connectionDurationMs: 0,
      averageRttMs: 0,
    },
    runtime: {
      runningTask: 0,
      workerCount: 0,
      schedulerQueue: 0,
      bufferUsage: 0,
      sessionCount: 0,
      runtimeUptimeSeconds: 0,
    },
    http: {
      requestsTotal: 0,
      activeRequests: 0,
      statusCodes: {},
      latency: {
        totalMs: 0,
        averageMs: 0,
      },
      bandwidth: {
        bytes: 0,
      },
    },
    tls: {
      sessionCount: 0,
      handshakeCount: 0,
      errorCount: 0,
      trafficBytes: 0,
    },
    system: {
      cpuUsage: 0,
      memoryUsage: 0,
      diskUsage: 0,
      threadCount: 0,
      processUptimeSeconds: 0,
      openFile: 0,
    },
    client: {
      onlineTimeSeconds: 0,
      openProject: 0,
      currentWorkspace: '',
      uiFps: 0,
      memoryBytes: 0,
    },
  }

  return {
    overview: {
      tunnelCount: 0,
      runningTunnel: 0,
      currentConnection: 0,
      todayTraffic: 0,
      totalTraffic: 0,
      averageRttMs: 0,
      runtimeUptimeSeconds: 0,
      healthScore: 0,
    },
    statistics,
    realtimeSpeed: [],
    connectionTrend: [],
    trafficTrend: [],
    tunnelStatus: [],
    serverStatus: [],
    systemHealth: {
      overall: 'offline',
      signals: [],
      updatedAt: now,
    },
    tunnels: [],
    recentActivity: [],
    visualSummary: {
      metricCards: [
        { key: 'totalTunnels', icon: 'router', tone: 'primary' },
        { key: 'onlineTunnels', icon: 'check-circle', tone: 'success' },
        { key: 'activeConnections', icon: 'users', tone: 'secondary' },
        { key: 'traffic', icon: 'activity', tone: 'info' },
        { key: 'latency', icon: 'clock', tone: 'warning' },
        { key: 'runtimeUptime', icon: 'shield-check', tone: 'healthy' },
      ],
      tunnelState: {
        running: 0,
        warning: 0,
        stopped: 0,
        runningRate: 0,
        warningRate: 0,
        stoppedRate: 0,
      },
      protocolDistribution: [],
      requestBuckets: [],
      errorBuckets: [],
      requestTotal: 0,
      errorTotal: 0,
    },
    generatedAt: now,
  }
}

class RuntimeStatisticsService implements StatisticsService {
  async getStatistics() {
    if (!isTauri()) {
      throw runtimeUnavailableError('runtime_get_statistics')
    }

    return ipc.invoke<Statistics>('runtime_get_statistics')
  }

  async getTraffic() {
    return (await this.getStatistics()).traffic
  }
}

class RuntimeMetricsService implements MetricsService {
  async collect() {
    if (!isTauri()) {
      throw runtimeUnavailableError('runtime_collect_metrics')
    }

    return ipc.invoke<Metric[]>('runtime_collect_metrics')
  }
}

class RuntimeHealthService implements HealthService {
  async getHealthReport() {
    if (!isTauri()) {
      throw runtimeUnavailableError('runtime_get_health')
    }

    return ipc.invoke<HealthReport>('runtime_get_health')
  }
}

class RuntimeDashboardService implements DashboardService {
  private readonly listeners = new Set<(data: DashboardData) => void>()
  private readonly errorListeners = new Set<(error: unknown) => void>()
  private timer: number | undefined
  private inFlight: Promise<DashboardData> | undefined

  async getDashboard() {
    if (!isTauri()) {
      throw runtimeUnavailableError('runtime_get_dashboard')
    }

    // 监控刷新可能来自按钮、订阅和导出；复用同一个 IPC，避免慢请求乱序覆盖新数据。
    this.inFlight ??= ipc.invoke<DashboardData>('runtime_get_dashboard').finally(() => {
      this.inFlight = undefined
    })
    return this.inFlight
  }

  subscribe(listener: (data: DashboardData) => void, onError?: (error: unknown) => void) {
    this.listeners.add(listener)
    if (onError) {
      this.errorListeners.add(onError)
    }
    this.start()

    return () => {
      this.listeners.delete(listener)
      if (onError) {
        this.errorListeners.delete(onError)
      }
      if (this.listeners.size === 0) {
        this.stop()
      }
    }
  }

  private start() {
    if (this.timer !== undefined) return
    this.scheduleNext()
  }

  private stop() {
    if (this.timer === undefined) return
    window.clearTimeout(this.timer)
    this.timer = undefined
  }

  private scheduleNext() {
    if (this.timer !== undefined || this.listeners.size === 0) return
    this.timer = window.setTimeout(async () => {
      this.timer = undefined
      await this.publish()
      this.scheduleNext()
    }, 1000)
  }

  private async publish() {
    try {
      const data = await this.getDashboard()
      this.listeners.forEach((listener) => listener(data))
    } catch (error) {
      // 后台订阅刷新失败不能静默吞掉，避免发布环境隐藏 Runtime 断连问题。
      console.warn('监控数据后台刷新失败', error)
      this.errorListeners.forEach((listener) => listener(error))
    }
  }
}

class RuntimeExportService implements ExportService {
  async exportJson() {
    return JSON.stringify(await dashboardService.getDashboard(), null, 2)
  }

  async exportCsv() {
    const metrics = await metricsService.collect()
    const rows = ['name,kind,scope,unit,value,timestamp']
    for (const metric of metrics) {
      rows.push(
        [
          metric.name,
          metric.kind,
          metric.scope,
          metric.unit,
          Array.isArray(metric.value) ? metric.value.join('|') : metric.value,
          metric.timestamp,
        ].join(','),
      )
    }
    return rows.join('\n')
  }
}

export const statisticsService: StatisticsService = new RuntimeStatisticsService()
export const metricsService: MetricsService = new RuntimeMetricsService()
export const healthService: HealthService = new RuntimeHealthService()
export const dashboardService: DashboardService = new RuntimeDashboardService()
export const exportService: ExportService = new RuntimeExportService()

function runtimeUnavailableError(command: string) {
  return new GateAppError({
    code: 'RUNTIME_UNAVAILABLE',
    messageKey: 'errors.runtimeUnavailable',
    details: { command },
    timestamp: Date.now(),
  })
}
