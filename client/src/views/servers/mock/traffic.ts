/* ==================================================================
   Server Mock — 流量数据
   ------------------------------------------------------------------
   生成服务器流量采样与统计。全部 Mock。
   ================================================================== */

import type { ServerTraffic, ServerTrafficPoint } from "../types"

const now = Date.now()
const min = 60 * 1000

function rand(min: number, max: number): number {
  return Math.floor(Math.random() * (max - min + 1)) + min
}

/* ── 生成历史流量采样 ── */
export function makeTrafficHistory(base: number): ServerTrafficPoint[] {
  const points: ServerTrafficPoint[] = []
  for (let i = 11; i >= 0; i--) {
    const d = new Date(now - i * 5 * min)
    const label = `${String(d.getHours()).padStart(2, "0")}:${String(d.getMinutes()).padStart(2, "0")}`
    const wave = Math.sin(i / 2) * 0.4 + 0.6
    points.push({
      time: label,
      upload: Math.floor(base * wave * (0.4 + Math.random() * 0.6)),
      download: Math.floor(base * wave * 1.6 * (0.4 + Math.random() * 0.6)),
    })
  }
  return points
}

/* ── 生成流量统计 ── */
export function makeTraffic(running: boolean): ServerTraffic {
  const baseSpeed = running ? rand(50, 512) * 1024 : 0
  const upSpeed = running ? Math.floor(baseSpeed * (0.3 + Math.random() * 0.5)) : 0
  const downSpeed = running ? baseSpeed : 0

  const totalUp = running ? rand(50, 8000) * 1024 * 1024 : rand(0, 10) * 1024 * 1024
  const totalDown = running ? rand(100, 20000) * 1024 * 1024 : rand(0, 20) * 1024 * 1024

  return {
    uploadSpeed: upSpeed,
    downloadSpeed: downSpeed,
    totalUpload: totalUp,
    totalDownload: totalDown,
    todayUpload: Math.floor(totalUp * (0.05 + Math.random() * 0.15)),
    todayDownload: Math.floor(totalDown * (0.05 + Math.random() * 0.15)),
    weekUpload: Math.floor(totalUp * (0.4 + Math.random() * 0.3)),
    weekDownload: Math.floor(totalDown * (0.4 + Math.random() * 0.3)),
    monthUpload: Math.floor(totalUp * (0.8 + Math.random() * 0.2)),
    monthDownload: Math.floor(totalDown * (0.8 + Math.random() * 0.2)),
    history: makeTrafficHistory(baseSpeed),
  }
}
