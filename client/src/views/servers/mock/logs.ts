/* ==================================================================
   Server Mock — 日志数据
   ------------------------------------------------------------------
   生成服务器日志列表。Console 风格。全部 Mock。
   ================================================================== */

import type { ServerConnection, ServerLog, ServerLogLevel } from "../types"

const now = Date.now()

function rand(min: number, max: number): number {
  return Math.floor(Math.random() * (max - min + 1)) + min
}

function pick<T>(arr: T[]): T {
  return arr[rand(0, arr.length - 1)]
}

function randomIp(): string {
  return `${rand(10, 220)}.${rand(0, 255)}.${rand(0, 255)}.${rand(1, 254)}`
}

/* ── 日志模板 ── */
const logTemplates: Array<{ level: ServerLogLevel; source: string; msg: string }> = [
  { level: "info", source: "gateway", msg: "client connected from remote" },
  { level: "info", source: "transport", msg: "heartbeat received, connection healthy" },
  { level: "success", source: "auth", msg: "token validated successfully" },
  { level: "info", source: "tunnel", msg: "new tunnel registered on port 8080" },
  { level: "debug", source: "transport", msg: "packet forwarded to upstream" },
  { level: "warn", source: "transport", msg: "high latency detected, 320ms" },
  { level: "info", source: "gateway", msg: "request routed to project service" },
  { level: "error", source: "auth", msg: "invalid token, access denied" },
  { level: "debug", source: "gateway", msg: "cache hit for static resource" },
  { level: "info", source: "gateway", msg: "client disconnected gracefully" },
  { level: "success", source: "tunnel", msg: "tunnel closed, resources released" },
  { level: "warn", source: "system", msg: "disk usage above 80%" },
  { level: "info", source: "system", msg: "scheduled health check completed" },
  { level: "error", source: "transport", msg: "connection reset by remote peer" },
  { level: "debug", source: "gateway", msg: "rate limiter applied, 100 req/s" },
]

/* ── 生成日志列表 ── */
export function makeLogs(count: number): ServerLog[] {
  const logs: ServerLog[] = []
  for (let i = 0; i < count; i++) {
    const tpl = pick(logTemplates)
    logs.push({
      id: `slog-${i}-${Math.random().toString(36).slice(2, 6)}`,
      level: tpl.level,
      message: tpl.msg,
      timestamp: now - (count - i) * rand(2000, 30000),
      source: tpl.source,
    })
  }
  return logs
}

/* ── 区域池（连接 Mock） ── */
const regions = [
  "Tokyo, JP",
  "Singapore, SG",
  "Frankfurt, DE",
  "San Jose, US",
  "Seoul, KR",
  "London, GB",
  "Sydney, AU",
  "São Paulo, BR",
]

/* ── 生成连接列表 ── */
export function makeConnections(count: number): ServerConnection[] {
  const list: ServerConnection[] = []
  for (let i = 0; i < count; i++) {
    const startedAt = new Date(now - rand(5, 3600) * 1000).toISOString()
    list.push({
      id: `sconn-${i}-${Math.random().toString(36).slice(2, 6)}`,
      clientIp: randomIp(),
      region: pick(regions),
      duration: rand(5, 3600),
      status: i < count - 1 ? "active" : "idle",
      protocol: pick(["http", "tcp", "ws"]),
      startedAt,
    })
  }
  return list
}
