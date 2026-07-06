import { mockDashboard, mockHealth, mockMetrics, mockStatistics } from "../mock"
import type {
  DashboardData,
  HealthReport,
  Metric,
  Statistics,
  TrafficStatistics,
} from "../types"

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

/** Mock implementation of StatisticsService. */
export class MockStatisticsService implements StatisticsService {
  async getStatistics() {
    return mockStatistics.snapshot()
  }

  async getTraffic() {
    return mockStatistics.snapshot().traffic
  }
}

/** Mock implementation of MetricsService. */
export class MockMetricsService implements MetricsService {
  async collect() {
    return mockMetrics.collect()
  }
}

/** Mock implementation of HealthService. */
export class MockHealthService implements HealthService {
  async getHealthReport() {
    const statistics = mockStatistics.snapshot()
    return mockHealth.report(statistics)
  }
}

/** Mock implementation of DashboardService. */
export class MockDashboardService implements DashboardService {
  async getDashboard() {
    return mockDashboard.snapshot()
  }

  subscribe(listener: (data: DashboardData) => void) {
    return mockDashboard.subscribe(listener)
  }
}

/** Mock implementation of ExportService. */
export class MockExportService implements ExportService {
  async exportJson() {
    return JSON.stringify(mockDashboard.snapshot(), null, 2)
  }

  async exportCsv() {
    const metrics = mockMetrics.collect()
    const rows = ["name,kind,scope,unit,value,timestamp"]
    for (const metric of metrics) {
      rows.push(
        [
          metric.name,
          metric.kind,
          metric.scope,
          metric.unit,
          Array.isArray(metric.value) ? metric.value.join("|") : metric.value,
          metric.timestamp,
        ].join(","),
      )
    }
    return rows.join("\n")
  }
}

export const statisticsService: StatisticsService = new MockStatisticsService()
export const metricsService: MetricsService = new MockMetricsService()
export const healthService: HealthService = new MockHealthService()
export const dashboardService: DashboardService = new MockDashboardService()
export const exportService: ExportService = new MockExportService()
