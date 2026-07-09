import { TauriIpcClient } from '@/ipc'
import type { DashboardData, HealthReport, Metric, Statistics, TrafficStatistics } from '../types'

const ipc = new TauriIpcClient()

/** Client-side statistics service contract. */
export interface StatisticsService {
  getStatistics(): Promise<Statistics>
  getTraffic(): Promise<TrafficStatistics>
}

/** Client-side metrics service contract. */
export interface MetricsService {
  collect(): Promise<Metric[]>
}

/** Client-side health service contract. */
export interface HealthService {
  getHealthReport(): Promise<HealthReport>
}

/** Client-side dashboard service contract. */
export interface DashboardService {
  getDashboard(): Promise<DashboardData>
  subscribe(listener: (data: DashboardData) => void): () => void
}

/** Client-side export service contract. */
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
    generatedAt: now,
  }
}

class RuntimeStatisticsService implements StatisticsService {
  async getStatistics() {
    return ipc.invoke<Statistics>('runtime_get_statistics')
  }

  async getTraffic() {
    return (await this.getStatistics()).traffic
  }
}

class RuntimeMetricsService implements MetricsService {
  async collect() {
    return ipc.invoke<Metric[]>('runtime_collect_metrics')
  }
}

class RuntimeHealthService implements HealthService {
  async getHealthReport() {
    return ipc.invoke<HealthReport>('runtime_get_health')
  }
}

class RuntimeDashboardService implements DashboardService {
  private readonly listeners = new Set<(data: DashboardData) => void>()
  private timer: number | undefined

  async getDashboard() {
    return ipc.invoke<DashboardData>('runtime_get_dashboard')
  }

  subscribe(listener: (data: DashboardData) => void) {
    this.listeners.add(listener)
    void this.publish()
    this.start()

    return () => {
      this.listeners.delete(listener)
      if (this.listeners.size === 0) {
        this.stop()
      }
    }
  }

  private start() {
    if (this.timer !== undefined) return
    this.timer = window.setInterval(() => {
      void this.publish()
    }, 1000)
  }

  private stop() {
    if (this.timer === undefined) return
    window.clearInterval(this.timer)
    this.timer = undefined
  }

  private async publish() {
    try {
      const data = await this.getDashboard()
      this.listeners.forEach((listener) => listener(data))
    } catch {
      // Refresh errors are surfaced by explicit refresh calls in the composable.
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
