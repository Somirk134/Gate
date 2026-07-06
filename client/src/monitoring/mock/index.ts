import type {
  ClientStatistics,
  ConnectionStatistics,
  ConnectionTrendPoint,
  DashboardData,
  DashboardTunnel,
  HealthReport,
  HealthSignal,
  HealthStatus,
  Metric,
  RealtimeSpeedPoint,
  RuntimeStatistics,
  Statistics,
  SystemStatistics,
  TrafficStatistics,
  TrafficTrendPoint,
  TunnelStatistics,
} from "../types"

const KB = 1024
const MB = KB * 1024
const GB = MB * 1024
const startTime = Date.now() - 1000 * 60 * 42

function clamp(value: number, min: number, max: number) {
  return Math.min(max, Math.max(min, value))
}

function wave(now: number, base: number, amplitude: number, speed = 9000) {
  const primary = Math.sin(now / speed) * amplitude
  const secondary = Math.cos(now / (speed * 1.9)) * amplitude * 0.38
  return base + primary + secondary
}

function statusFromScore(score: number): HealthStatus {
  if (score < 35) return "critical"
  if (score < 68) return "warning"
  return "healthy"
}

function trend<T>(count: number, stepMs: number, factory: (timestamp: number, index: number) => T) {
  const now = Date.now()
  return Array.from({ length: count }, (_, index) =>
    factory(now - (count - index - 1) * stepMs, index),
  )
}

/** Mock traffic generator. */
export class MockTraffic {
  snapshot(now = Date.now()): TrafficStatistics {
    const uploadSpeedBps = clamp(wave(now, 1.4 * MB, 0.9 * MB), 128 * KB, 4.2 * MB)
    const downloadSpeedBps = clamp(wave(now + 2500, 3.8 * MB, 1.7 * MB), 256 * KB, 8.6 * MB)
    const totalTrafficBytes = 184 * GB + Math.floor((now - startTime) / 1000) * 700 * KB

    return {
      uploadBytes: 42 * GB + Math.floor((now - startTime) / 1000) * 180 * KB,
      downloadBytes: 96 * GB + Math.floor((now - startTime) / 1000) * 520 * KB,
      uploadSpeedBps,
      downloadSpeedBps,
      peakSpeedBps: Math.max(uploadSpeedBps, downloadSpeedBps) * 1.34,
      averageSpeedBps: (uploadSpeedBps + downloadSpeedBps) / 2,
      todayTrafficBytes: 18.7 * GB + Math.floor((now - startTime) / 1000) * 520 * KB,
      totalTrafficBytes,
    }
  }

  realtime(count = 48): RealtimeSpeedPoint[] {
    return trend(count, 1000, (timestamp) => {
      const traffic = this.snapshot(timestamp)
      return {
        timestamp,
        uploadBps: traffic.uploadSpeedBps,
        downloadBps: traffic.downloadSpeedBps,
      }
    })
  }

  trend(count = 24): TrafficTrendPoint[] {
    return trend(count, 60 * 60 * 1000, (timestamp, index) => ({
      timestamp,
      uploadBytes: Math.floor((2.1 + index * 0.18) * GB + wave(timestamp, 0, 0.3 * GB)),
      downloadBytes: Math.floor((5.4 + index * 0.44) * GB + wave(timestamp + 1000, 0, 0.7 * GB)),
    }))
  }
}

/** Mock statistics generator. */
export class MockStatistics {
  private readonly traffic = new MockTraffic()

  snapshot(now = Date.now()): Statistics {
    const traffic = this.traffic.snapshot(now)
    const tunnel: TunnelStatistics = {
      tunnelCount: 32,
      runningTunnel: 21,
      stoppedTunnel: 11,
      upload: traffic.uploadBytes,
      download: traffic.downloadBytes,
      peakSpeedBps: traffic.peakSpeedBps,
      averageSpeedBps: traffic.averageSpeedBps,
      runningTimeSeconds: Math.floor((now - startTime) / 1000),
      todayTraffic: traffic.todayTrafficBytes,
      totalTraffic: traffic.totalTrafficBytes,
    }
    const connection: ConnectionStatistics = {
      currentConnection: Math.round(clamp(wave(now, 128, 46), 38, 220)),
      totalConnection: 19342 + Math.floor((now - startTime) / 450),
      success: 18880 + Math.floor((now - startTime) / 620),
      failure: Math.round(clamp(wave(now, 18, 8, 17000), 0, 42)),
      reconnect: Math.round(clamp(wave(now, 23, 12, 21000), 2, 54)),
      disconnect: Math.round(clamp(wave(now, 16, 9, 19000), 1, 38)),
      connectionDurationMs: 1000 * 60 * 18,
      averageRttMs: clamp(wave(now, 42, 11, 12000), 18, 92),
    }
    const runtime: RuntimeStatistics = {
      runningTask: Math.round(clamp(wave(now, 74, 16), 36, 120)),
      workerCount: 12,
      schedulerQueue: Math.round(clamp(wave(now, 18, 9, 7000), 0, 42)),
      bufferUsage: clamp(wave(now, 51, 18, 10000), 18, 84),
      sessionCount: Math.round(clamp(wave(now, 46, 14, 8000), 18, 80)),
      runtimeUptimeSeconds: Math.floor((now - startTime) / 1000),
    }
    const system: SystemStatistics = {
      cpuUsage: clamp(wave(now, 44, 17, 11000), 12, 88),
      memoryUsage: clamp(wave(now + 1500, 62, 13, 16000), 28, 91),
      diskUsage: clamp(wave(now, 48, 4, 30000), 34, 69),
      threadCount: 42,
      processUptimeSeconds: Math.floor((now - startTime) / 1000),
      openFile: 126,
    }
    const client: ClientStatistics = {
      onlineTimeSeconds: Math.floor((now - startTime) / 1000),
      openProject: 7,
      currentWorkspace: "Gate / Production Preview",
      uiFps: clamp(wave(now, 58, 2, 5000), 52, 60),
      memoryBytes: 418 * MB,
    }

    return {
      collectedAt: now,
      tunnel,
      traffic,
      connection,
      runtime,
      system,
      client,
    }
  }

  connectionTrend(count = 36): ConnectionTrendPoint[] {
    return trend(count, 60 * 1000, (timestamp) => {
      const connection = this.snapshot(timestamp).connection
      return {
        timestamp,
        current: connection.currentConnection,
        success: Math.floor(connection.success / 100),
        failure: connection.failure,
        reconnect: connection.reconnect,
      }
    })
  }
}

/** Mock metric source. */
export class MockMetrics {
  private readonly statistics = new MockStatistics()

  collect(now = Date.now()): Metric[] {
    const snapshot = this.statistics.snapshot(now)
    return [
      {
        name: "gate.system.cpu.usage",
        description: "CPU usage",
        kind: "gauge",
        scope: "system",
        unit: "percent",
        value: snapshot.system.cpuUsage,
        labels: [],
        timestamp: now,
      },
      {
        name: "gate.system.memory.usage",
        description: "Memory usage",
        kind: "gauge",
        scope: "system",
        unit: "percent",
        value: snapshot.system.memoryUsage,
        labels: [],
        timestamp: now,
      },
      {
        name: "gate.traffic.download.bps",
        description: "Realtime download speed",
        kind: "rate",
        scope: "network",
        unit: "bytes_per_second",
        value: snapshot.traffic.downloadSpeedBps,
        labels: [],
        timestamp: now,
      },
    ]
  }
}

/** Mock health source. */
export class MockHealth {
  report(statistics: Statistics): HealthReport {
    const cpuScore = clamp(100 - statistics.system.cpuUsage * 0.55, 0, 100)
    const memoryScore = clamp(100 - statistics.system.memoryUsage * 0.48, 0, 100)
    const connectionScore = clamp(100 - statistics.connection.failure * 1.2, 0, 100)
    const runtimeScore = clamp(100 - statistics.runtime.bufferUsage * 0.42, 0, 100)
    const signals: HealthSignal[] = [
      {
        target: "system",
        status: statusFromScore(Math.min(cpuScore, memoryScore)),
        message: "CPU and memory remain within configured guardrails",
        score: Math.min(cpuScore, memoryScore),
        timestamp: statistics.collectedAt,
      },
      {
        target: "connection",
        status: statusFromScore(connectionScore),
        message: "Connection failures and reconnects are being watched",
        score: connectionScore,
        timestamp: statistics.collectedAt,
      },
      {
        target: "runtime",
        status: statusFromScore(runtimeScore),
        message: "Runtime queue and buffer pressure are stable",
        score: runtimeScore,
        timestamp: statistics.collectedAt,
      },
      {
        target: "heartbeat",
        status: "healthy",
        message: "Heartbeat loop is active",
        score: 96,
        timestamp: statistics.collectedAt,
      },
      {
        target: "server",
        status: "healthy",
        message: "Primary relay is online",
        score: 94,
        timestamp: statistics.collectedAt,
      },
      {
        target: "tunnel",
        status: "healthy",
        message: "Running tunnels are accepting traffic",
        score: 91,
        timestamp: statistics.collectedAt,
      },
    ]
    const score = Math.min(...signals.map((signal) => signal.score))
    return {
      overall: statusFromScore(score),
      signals,
      updatedAt: statistics.collectedAt,
    }
  }
}

/** Mock dashboard source with auto refresh support. */
export class MockDashboard {
  private readonly statistics = new MockStatistics()
  private readonly traffic = new MockTraffic()
  private readonly health = new MockHealth()
  private listeners = new Set<(data: DashboardData) => void>()
  private timer?: number

  snapshot(now = Date.now()): DashboardData {
    const statistics = this.statistics.snapshot(now)
    const systemHealth = this.health.report(statistics)
    const healthScore = Math.min(...systemHealth.signals.map((signal) => signal.score))
    return {
      overview: {
        tunnelCount: statistics.tunnel.tunnelCount,
        runningTunnel: statistics.tunnel.runningTunnel,
        currentConnection: statistics.connection.currentConnection,
        todayTraffic: statistics.tunnel.todayTraffic,
        totalTraffic: statistics.tunnel.totalTraffic,
        averageRttMs: statistics.connection.averageRttMs,
        runtimeUptimeSeconds: statistics.runtime.runtimeUptimeSeconds,
        healthScore,
      },
      statistics,
      realtimeSpeed: this.traffic.realtime(),
      connectionTrend: this.statistics.connectionTrend(),
      trafficTrend: this.traffic.trend(),
      tunnelStatus: [
        { label: "Running", count: statistics.tunnel.runningTunnel },
        { label: "Stopped", count: statistics.tunnel.stoppedTunnel },
        { label: "Warning", count: 2 },
      ],
      serverStatus: [
        { label: "Online", count: 4 },
        { label: "Warning", count: 1 },
        { label: "Offline", count: 1 },
      ],
      systemHealth,
      tunnels: this.tunnels(now),
      recentActivity: [
        { id: "a1", title: "Metrics collected from mock.system", category: "collector", timestamp: now - 12_000 },
        { id: "a2", title: "Realtime aggregation window refreshed", category: "aggregation", timestamp: now - 25_000 },
        { id: "a3", title: "Health report recalculated", category: "health", timestamp: now - 42_000 },
        { id: "a4", title: "Dashboard payload generated", category: "dashboard", timestamp: now - 58_000 },
      ],
      generatedAt: now,
    }
  }

  subscribe(listener: (data: DashboardData) => void) {
    this.listeners.add(listener)
    listener(this.snapshot())
    return () => {
      this.listeners.delete(listener)
    }
  }

  start(intervalMs = 1000) {
    if (this.timer) return
    this.timer = window.setInterval(() => {
      const data = this.snapshot()
      this.listeners.forEach((listener) => listener(data))
    }, intervalMs)
  }

  stop() {
    if (!this.timer) return
    window.clearInterval(this.timer)
    this.timer = undefined
  }

  private tunnels(now: number): DashboardTunnel[] {
    return [
      {
        id: "tun-api",
        name: "api-gateway",
        protocol: "https",
        status: "running",
        uploadSpeedBps: clamp(wave(now, 0.9 * MB, 0.4 * MB), 80 * KB, 2 * MB),
        downloadSpeedBps: clamp(wave(now + 300, 2.4 * MB, 0.9 * MB), 120 * KB, 4.4 * MB),
        connections: 42,
        uptimeSeconds: Math.floor((now - startTime) / 1000),
      },
      {
        id: "tun-web",
        name: "web-preview",
        protocol: "http",
        status: "running",
        uploadSpeedBps: clamp(wave(now, 320 * KB, 120 * KB), 40 * KB, 900 * KB),
        downloadSpeedBps: clamp(wave(now + 800, 720 * KB, 300 * KB), 60 * KB, 1.6 * MB),
        connections: 18,
        uptimeSeconds: Math.floor((now - startTime) / 1000) - 240,
      },
      {
        id: "tun-db",
        name: "postgres-admin",
        protocol: "tcp",
        status: "warning",
        uploadSpeedBps: clamp(wave(now, 180 * KB, 80 * KB), 10 * KB, 520 * KB),
        downloadSpeedBps: clamp(wave(now + 1000, 260 * KB, 100 * KB), 20 * KB, 700 * KB),
        connections: 7,
        uptimeSeconds: Math.floor((now - startTime) / 1000) - 720,
      },
      {
        id: "tun-ssh",
        name: "ops-ssh",
        protocol: "tcp",
        status: "running",
        uploadSpeedBps: clamp(wave(now, 96 * KB, 32 * KB), 8 * KB, 240 * KB),
        downloadSpeedBps: clamp(wave(now + 1900, 128 * KB, 48 * KB), 8 * KB, 320 * KB),
        connections: 3,
        uptimeSeconds: Math.floor((now - startTime) / 1000) - 1200,
      },
    ]
  }
}

export const mockStatistics = new MockStatistics()
export const mockTraffic = new MockTraffic()
export const mockMetrics = new MockMetrics()
export const mockHealth = new MockHealth()
export const mockDashboard = new MockDashboard()
