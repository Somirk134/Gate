/* ==================================================================
   Server Mock — 监控数据
   ------------------------------------------------------------------
   生成服务器资源监控（CPU / Memory / Disk / Load / Network / Traffic / Connection）。
   全部 Mock。
   ================================================================== */

import type {
  ServerConnectionMetric,
  ServerLoadMetric,
  ServerMonitor,
  ServerMonitorPoint,
  ServerNetworkMetric,
  ServerResourceMetric,
} from "../types"

function rand(min: number, max: number): number {
  return Math.floor(Math.random() * (max - min + 1)) + min
}

/* ── 生成百分比采样 ── */
export function makePercentHistory(base: number, count = 12): number[] {
  const arr: number[] = []
  for (let i = 0; i < count; i++) {
    const v = base + (Math.random() - 0.5) * 20
    arr.push(Math.max(2, Math.min(98, Math.round(v))))
  }
  return arr
}

/* ── 生成资源指标 ── */
export function makeResourceMetric(
  percent: number,
  total: number,
  unit: "GB" | "MB",
): ServerResourceMetric {
  return {
    percent,
    used: Number((total * (percent / 100)).toFixed(1)),
    total,
    unit,
    history: makePercentHistory(percent),
  }
}

/* ── 生成负载 ── */
export function makeLoad(cores: number): ServerLoadMetric {
  return {
    load1: Number((0.4 + Math.random() * cores * 0.8).toFixed(2)),
    load5: Number((0.3 + Math.random() * cores * 0.6).toFixed(2)),
    load15: Number((0.2 + Math.random() * cores * 0.4).toFixed(2)),
    cores,
  }
}

/* ── 生成网络监控点 ── */
export function makeMonitorHistory(base: number): ServerMonitorPoint[] {
  const points: ServerMonitorPoint[] = []
  const now = Date.now()
  for (let i = 11; i >= 0; i--) {
    const d = new Date(now - i * 5 * 60 * 1000)
    const label = `${String(d.getHours()).padStart(2, "0")}:${String(d.getMinutes()).padStart(2, "0")}`
    const wave = Math.sin(i / 2) * 0.3 + 0.7
    points.push({
      time: label,
      cpu: rand(10, 80),
      memory: rand(30, 70),
      network: Math.floor(base * wave),
    })
  }
  return points
}

/* ── 生成网络指标 ── */
export function makeNetworkMetric(running: boolean): ServerNetworkMetric {
  const baseSpeed = running ? rand(50, 512) * 1024 : 0
  return {
    uploadSpeed: running ? Math.floor(baseSpeed * 0.4) : 0,
    downloadSpeed: running ? baseSpeed : 0,
    totalUpload: running ? rand(50, 8000) * 1024 * 1024 : 0,
    totalDownload: running ? rand(100, 20000) * 1024 * 1024 : 0,
    history: makeMonitorHistory(baseSpeed),
  }
}

/* ── 生成连接指标 ── */
export function makeConnectionMetric(running: boolean): ServerConnectionMetric {
  return {
    active: running ? rand(2, 64) : 0,
    total: rand(500, 80000),
    failed: rand(0, 120),
  }
}

/* ── 生成完整监控 ── */
export function makeMonitor(running: boolean, traffic: ServerMonitor["traffic"]): ServerMonitor {
  const cpuPercent = running ? rand(8, 75) : 0
  const memPercent = running ? rand(25, 78) : 0
  const diskPercent = rand(30, 70)

  return {
    cpu: makeResourceMetric(cpuPercent, 8, "GB"),
    memory: makeResourceMetric(memPercent, 16, "GB"),
    disk: makeResourceMetric(diskPercent, 500, "GB"),
    load: makeLoad(running ? rand(2, 16) : 4),
    network: makeNetworkMetric(running),
    traffic,
    connections: makeConnectionMetric(running),
  }
}
