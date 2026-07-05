/* ==================================================================
   Server Mock — 健康检查数据
   ------------------------------------------------------------------
   生成健康检查项与报告。全部 Mock。
   ================================================================== */

import type { ServerHealth, ServerHealthItem } from "../types"

const now = Date.now()

/* ── 健康检查项种子 ── */
export function buildHealthItems(
  online: boolean,
  version: string,
): ServerHealthItem[] {
  const items: ServerHealthItem[] = [
    {
      key: "online",
      label: "Server Online",
      status: online ? "pass" : "fail",
      message: online ? "服务器在线，响应正常" : "服务器无法访问",
      latency: online ? 24 : 0,
      icon: "servers",
    },
    {
      key: "api",
      label: "API Available",
      status: online ? "pass" : "fail",
      message: online ? "API 接口可用" : "API 接口无响应",
      latency: online ? 18 : 0,
      icon: "code",
    },
    {
      key: "token",
      label: "Token Valid",
      status: online ? "pass" : "warn",
      message: online ? "Token 有效，认证成功" : "Token 未能验证",
      latency: online ? 12 : 0,
      icon: "key",
    },
    {
      key: "tunnel",
      label: "Tunnel Service",
      status: online ? "pass" : "fail",
      message: online ? "Tunnel 服务运行中" : "Tunnel 服务未启动",
      latency: online ? 32 : 0,
      icon: "router",
    },
    {
      key: "disk",
      label: "Disk",
      status: "pass",
      message: "磁盘空间充足（剩余 65%）",
      latency: 8,
      icon: "hard-drive",
    },
    {
      key: "memory",
      label: "Memory",
      status: "pass",
      message: "内存使用正常",
      latency: 6,
      icon: "memory-stick",
    },
    {
      key: "clock",
      label: "Clock Sync",
      status: "pass",
      message: "系统时钟已同步（NTP）",
      latency: 4,
      icon: "clock",
    },
    {
      key: "version",
      label: "Version",
      status: "pass",
      message: `服务端版本 ${version} 兼容`,
      latency: 5,
      icon: "info-circle",
    },
  ]
  return items
}

/* ── 健康报告 ── */
export function buildHealth(
  online: boolean,
  version: string,
  scoreOverride?: number,
): ServerHealth {
  const items = buildHealthItems(online, version)
  const passCount = items.filter((i) => i.status === "pass").length
  const warnCount = items.filter((i) => i.status === "warn").length
  const failCount = items.filter((i) => i.status === "fail").length

  let overall: ServerHealth["overall"] = "healthy"
  if (failCount > 0) overall = "critical"
  else if (warnCount > 0) overall = "warning"

  const score = scoreOverride ?? Math.round((passCount / items.length) * 100)

  return {
    overall,
    score,
    checkedAt: now - Math.floor(Math.random() * 60000),
    items,
  }
}
