/* ==================================================================
   Server Mock — 统计数据
   ------------------------------------------------------------------
   生成服务器运行统计。全部 Mock。
   ================================================================== */

import type { ServerStatistics } from "../types"

function rand(min: number, max: number): number {
  return Math.floor(Math.random() * (max - min + 1)) + min
}

/* ── 生成运行统计 ── */
export function makeStatistics(
  running: boolean,
  tunnelCount: number,
  projectCount: number,
): ServerStatistics {
  return {
    uptime: running ? rand(3600, 3600 * 24 * 30) : 0,
    tunnelCount,
    projectCount,
    totalConnections: rand(500, 100000),
    requests: rand(5000, 9999999),
    avgPing: running ? rand(8, 280) : 0,
    peakSpeed: running ? rand(512, 20480) * 1024 : 0,
  }
}
