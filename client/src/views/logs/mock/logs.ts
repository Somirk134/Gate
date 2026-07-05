import type { LogContext, LogItem, LogLevel, LogMetadata, LogSource } from "../types"
import { LOG_SOURCE_LIST } from "./sources"

const modules = [
  "gateway",
  "transport",
  "auth",
  "config",
  "tunnel.manager",
  "project.runner",
  "server.health",
  "statistics.collector",
  "updater",
  "plugin.host",
]
const projects = ["Atlas API", "Console Web", "IoT Gateway", "Monitoring Stack", "Billing Service"]
const tunnels = ["api-gateway", "web-frontend", "ssh-devbox", "postgres-db", "grafana-ui"]
const messages = [
  "connection accepted from remote client",
  "request forwarded to upstream service",
  "configuration snapshot loaded",
  "heartbeat packet sent",
  "permission check completed",
  "tunnel route table refreshed",
  "project process exited with code 0",
  "latency threshold exceeded",
  "authentication token expired",
  "failed to bind local port",
  "metrics sample persisted",
  "update manifest checked",
  "plugin lifecycle hook completed",
  "retrying websocket connection",
  "cache entry invalidated",
]

function pick<T>(items: T[], index: number): T {
  return items[index % items.length]
}

function weightedLevel(index: number): LogLevel {
  if (index % 97 === 0) return "FATAL"
  if (index % 13 === 0) return "ERROR"
  if (index % 7 === 0) return "WARN"
  if (index % 5 === 0) return "DEBUG"
  if (index % 11 === 0) return "TRACE"
  return "INFO"
}

function makeContext(index: number, source: LogSource): LogContext {
  return {
    environment: source === "SERVER" ? "server" : source === "CLIENT" ? "client" : "desktop",
    host: source === "SERVER" ? `edge-${(index % 4) + 1}` : "gate-desktop",
    processId: 4200 + (index % 400),
    thread: `worker-${(index % 12) + 1}`,
    sessionId: `ses_${(100000 + index * 17).toString(16)}`,
  }
}

function makeMetadata(index: number, source: LogSource): LogMetadata {
  return {
    durationMs: 8 + ((index * 37) % 1200),
    statusCode: index % 13 === 0 ? 502 : index % 7 === 0 ? 429 : 200,
    method: index % 3 === 0 ? "POST" : "GET",
    path: `/api/${source.toLowerCase()}/${(index % 9) + 1}`,
    ip: `10.${index % 24}.${(index * 3) % 255}.${(index * 7) % 255}`,
    bytesIn: 256 + index * 19,
    bytesOut: 1024 + index * 53,
    retry: index % 17 === 0 ? 2 : index % 9 === 0 ? 1 : 0,
    tags: [source.toLowerCase(), pick(["mock", "runtime", "network", "security"], index)],
  }
}

export function createMockLog(index: number, timestamp = Date.now()): LogItem {
  const source = pick(LOG_SOURCE_LIST, index)
  const level = weightedLevel(index)
  const projectName = source === "PROJECT" || index % 4 === 0 ? pick(projects, index) : undefined
  const tunnelName = source === "TUNNEL" || index % 5 === 0 ? pick(tunnels, index) : undefined
  const module = pick(modules, index)
  const message = `${pick(messages, index)} (${module})`
  const traceId = `trc_${(timestamp + index).toString(36)}`
  const requestId = index % 2 === 0 ? `req_${(timestamp - index).toString(36)}` : undefined

  const item: LogItem = {
    id: `log_${timestamp}_${index}`,
    timestamp,
    level,
    source,
    module,
    message,
    projectId: projectName ? `project-${projects.indexOf(projectName) + 1}` : undefined,
    projectName,
    tunnelId: tunnelName ? `tunnel-${tunnels.indexOf(tunnelName) + 1}` : undefined,
    tunnelName,
    traceId,
    requestId,
    context: makeContext(index, source),
    metadata: makeMetadata(index, source),
    stack: level === "ERROR" || level === "FATAL"
      ? [
          `at ${module}.execute (${module.replace(".", "/")}.ts:${24 + (index % 80)}:12)`,
          `at scheduler.flush (runtime/scheduler.ts:${10 + (index % 30)}:8)`,
          "at async dispatchLogEvent (runtime/logs.ts:42:5)",
        ]
      : undefined,
    raw: "",
  }

  item.raw = JSON.stringify(
    {
      time: new Date(item.timestamp).toISOString(),
      level: item.level,
      source: item.source,
      module: item.module,
      message: item.message,
      traceId: item.traceId,
      requestId: item.requestId,
      context: item.context,
      metadata: item.metadata,
    },
    null,
    2,
  )

  return item
}

export function generateMockLogs(count = 1000): LogItem[] {
  const now = Date.now()
  return Array.from({ length: count }, (_, index) => {
    const minutesAgo = count - index
    const jitter = (index % 60) * 1000
    return createMockLog(index + 1, now - minutesAgo * 60_000 - jitter)
  })
}

export const mockLogs = generateMockLogs(1000)
