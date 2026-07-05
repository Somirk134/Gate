/* ==================================================================
   Tunnel Mock 数据
   ------------------------------------------------------------------
   统一 Mock 数据源。所有 Tunnel 组件不写死数据，统一从此引入。
   生成 30 个不同协议/状态/服务器/项目/流量/连接数的隧道。

   后续替换为真实 Tunnel Engine 时，只需将 store 的 load 动作改为
   API 调用，并保持返回类型与 types/index.ts 一致即可无缝迁移。
   ================================================================== */

import type {
  Tunnel,
  TunnelConnection,
  TunnelLog,
  TunnelProtocol,
  TunnelStatus,
  TunnelTrafficPoint,
} from "../types"

const now = Date.now()
const min = 60 * 1000
const hour = 60 * min
const day = 24 * hour

/* ── 服务器池 ── */
export const mockServerNames = [
  "Tokyo Edge",
  "Frankfurt Relay",
  "US-West Gateway",
  "Singapore Hub",
  "Seoul Node",
]

/* 服务器域名（用于公网地址） */
const serverDomains: Record<string, string> = {
  "Tokyo Edge": "gate.jp",
  "Frankfurt Relay": "gate.eu",
  "US-West Gateway": "gate.us",
  "Singapore Hub": "gate.sg",
  "Seoul Node": "gate.kr",
}

/* ── 项目池 ── */
export const mockProjects = [
  { id: "p1", name: "My API Service" },
  { id: "p2", name: "Web App Frontend" },
  { id: "p3", name: "Dev Environment" },
  { id: "p4", name: "Monitoring Stack" },
  { id: "p5", name: "IoT Gateway" },
  { id: "p6", name: "E-Commerce Platform" },
  { id: "p7", name: "Database Cluster" },
  { id: "p8", name: "SSH Bastion" },
  { id: "p9", name: "Realtime Chat" },
  { id: "p10", name: "API Gateway" },
]

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

/* ── 客户端 IP 池 ── */
function randomIp(): string {
  return `${rand(10, 220)}.${rand(0, 255)}.${rand(0, 255)}.${rand(1, 254)}`
}

function rand(min: number, max: number): number {
  return Math.floor(Math.random() * (max - min + 1)) + min
}

function pick<T>(arr: T[]): T {
  return arr[rand(0, arr.length - 1)]
}

/* ── 生成历史流量采样 ── */
function makeHistory(base: number): TunnelTrafficPoint[] {
  const points: TunnelTrafficPoint[] = []
  for (let i = 11; i >= 0; i--) {
    const d = new Date(now - i * 5 * min)
    const label = `${String(d.getHours()).padStart(2, "0")}:${String(d.getMinutes()).padStart(2, "0")}`
    const wave = Math.sin(i / 2) * 0.4 + 0.6
    points.push({
      time: label,
      upload: Math.floor(base * wave * (0.5 + Math.random())),
      download: Math.floor(base * wave * 1.4 * (0.5 + Math.random())),
    })
  }
  return points
}

/* ── 生成连接列表 ── */
function makeConnections(count: number, protocol: TunnelProtocol): TunnelConnection[] {
  const list: TunnelConnection[] = []
  for (let i = 0; i < count; i++) {
    const startedAt = new Date(now - rand(5, 3600) * 1000).toISOString()
    list.push({
      id: `conn-${i}-${Math.random().toString(36).slice(2, 6)}`,
      clientIp: randomIp(),
      region: pick(regions),
      duration: rand(5, 3600),
      status: i < count - 1 ? "active" : "idle",
      protocol,
      startedAt,
    })
  }
  return list
}

/* ── 生成日志列表 ── */
const logTemplates: Array<{ level: TunnelLog["level"]; source: string; msg: string }> = [
  { level: "info", source: "frpc", msg: "login to server success" },
  { level: "info", source: "transport", msg: "tunnel established, public address ready" },
  { level: "success", source: "frpc", msg: "proxy started successfully" },
  { level: "info", source: "api", msg: "new connection from client" },
  { level: "debug", source: "transport", msg: "heartbeat packet sent" },
  { level: "warn", source: "transport", msg: "slow connection detected, latency 320ms" },
  { level: "info", source: "api", msg: "connection closed by peer" },
  { level: "error", source: "frpc", msg: "connection reset by remote server" },
  { level: "debug", source: "api", msg: "request forwarded to local service" },
  { level: "info", source: "frpc", msg: "reconnect attempt succeeded" },
  { level: "success", source: "transport", msg: "bandwidth upgraded to new tier" },
  { level: "warn", source: "frpc", msg: "port already in use, fallback applied" },
]

function makeLogs(count: number): TunnelLog[] {
  const logs: TunnelLog[] = []
  for (let i = 0; i < count; i++) {
    const tpl = pick(logTemplates)
    logs.push({
      id: `log-${i}-${Math.random().toString(36).slice(2, 6)}`,
      level: tpl.level,
      message: tpl.msg,
      timestamp: now - (count - i) * rand(2000, 30000),
      source: tpl.source,
    })
  }
  return logs
}

/* ── 状态分布权重 ── */
const statusPool: TunnelStatus[] = [
  "running", "running", "running", "running", "running", "running", "running",
  "running", "running", "running", "running", "running",
  "starting", "starting",
  "connecting", "connecting",
  "stopped", "stopped", "stopped", "stopped", "stopped",
  "error", "error",
  "disconnected",
  "restarting",
  "stopping",
  "offline",
]

/* ── 隧道种子 ── */
const tunnelSeeds: Array<{
  name: string
  protocol: TunnelProtocol
  localPort: number
  remotePort: number
  serverName: string
  projectName: string
  projectId: string
  tags: string[]
  remark: string
  autoStart: boolean
}> = [
  { name: "api-gateway", protocol: "http", localPort: 3000, remotePort: 8080, serverName: "Tokyo Edge", projectName: "My API Service", projectId: "p1", tags: ["API", "Production"], remark: "主 API 网关入口", autoStart: true },
  { name: "web-frontend", protocol: "http", localPort: 5173, remotePort: 80, serverName: "Tokyo Edge", projectName: "Web App Frontend", projectId: "p2", tags: ["Frontend", "Staging"], remark: "Vite 开发服务器", autoStart: false },
  { name: "postgres-db", protocol: "tcp", localPort: 5432, remotePort: 5432, serverName: "Tokyo Edge", projectName: "Database Cluster", projectId: "p7", tags: ["Database", "Production"], remark: "PostgreSQL 主库", autoStart: true },
  { name: "redis-cache", protocol: "tcp", localPort: 6379, remotePort: 6379, serverName: "Frankfurt Relay", projectName: "Database Cluster", projectId: "p7", tags: ["Database"], remark: "Redis 缓存层", autoStart: true },
  { name: "admin-panel", protocol: "http", localPort: 3001, remotePort: 3000, serverName: "Singapore Hub", projectName: "My API Service", projectId: "p1", tags: ["API", "Staging"], remark: "管理后台", autoStart: false },
  { name: "ssh-tunnel", protocol: "tcp", localPort: 22, remotePort: 2222, serverName: "Frankfurt Relay", projectName: "SSH Bastion", projectId: "p8", tags: ["SSH", "Production"], remark: "SSH 跳板入口", autoStart: true },
  { name: "grafana-ui", protocol: "http", localPort: 9000, remotePort: 8081, serverName: "Frankfurt Relay", projectName: "Monitoring Stack", projectId: "p4", tags: ["Demo"], remark: "Grafana 监控面板", autoStart: false },
  { name: "prometheus", protocol: "http", localPort: 9090, remotePort: 9090, serverName: "Frankfurt Relay", projectName: "Monitoring Stack", projectId: "p4", tags: ["Demo"], remark: "Prometheus 指标采集", autoStart: false },
  { name: "websocket-chat", protocol: "tcp", localPort: 4000, remotePort: 4000, serverName: "Singapore Hub", projectName: "Realtime Chat", projectId: "p9", tags: ["API", "Staging"], remark: "WebSocket 即时通讯", autoStart: false },
  { name: "mqtt-broker", protocol: "tcp", localPort: 1883, remotePort: 1883, serverName: "Seoul Node", projectName: "IoT Gateway", projectId: "p5", tags: ["API", "Production"], remark: "MQTT 物联网网关", autoStart: true },
  { name: "mongo-db", protocol: "tcp", localPort: 27017, remotePort: 27017, serverName: "Tokyo Edge", projectName: "Database Cluster", projectId: "p7", tags: ["Database"], remark: "MongoDB 分片集群", autoStart: true },
  { name: "kong-gateway", protocol: "http", localPort: 8000, remotePort: 8082, serverName: "Tokyo Edge", projectName: "API Gateway", projectId: "p10", tags: ["API", "Production"], remark: "Kong API 网关", autoStart: true },
  { name: "nginx-static", protocol: "http", localPort: 8080, remotePort: 8083, serverName: "US-West Gateway", projectName: "Web App Frontend", projectId: "p2", tags: ["Frontend"], remark: "Nginx 静态资源", autoStart: false },
  { name: "mysql-db", protocol: "tcp", localPort: 3306, remotePort: 3306, serverName: "Singapore Hub", projectName: "Database Cluster", projectId: "p7", tags: ["Database", "Production"], remark: "MySQL 主从", autoStart: true },
  { name: "mail-smtp", protocol: "tcp", localPort: 25, remotePort: 2525, serverName: "Frankfurt Relay", projectName: "My API Service", projectId: "p1", tags: ["API"], remark: "SMTP 邮件服务", autoStart: false },
  { name: "dev-server", protocol: "http", localPort: 4200, remotePort: 8084, serverName: "Singapore Hub", projectName: "Dev Environment", projectId: "p3", tags: ["Frontend", "Personal"], remark: "Angular 开发服务器", autoStart: false },
  { name: "elastic-search", protocol: "tcp", localPort: 9200, remotePort: 9200, serverName: "Frankfurt Relay", projectName: "Database Cluster", projectId: "p7", tags: ["Database"], remark: "Elasticsearch 搜索引擎", autoStart: true },
  { name: "rabbitmq-ui", protocol: "http", localPort: 15672, remotePort: 8085, serverName: "Seoul Node", projectName: "Dev Environment", projectId: "p3", tags: ["Demo"], remark: "RabbitMQ 管理界面", autoStart: false },
  { name: "shop-api", protocol: "http", localPort: 5000, remotePort: 8086, serverName: "Tokyo Edge", projectName: "E-Commerce Platform", projectId: "p6", tags: ["API", "Production"], remark: "电商核心 API", autoStart: true },
  { name: "shop-admin", protocol: "http", localPort: 5001, remotePort: 8087, serverName: "Tokyo Edge", projectName: "E-Commerce Platform", projectId: "p6", tags: ["Frontend", "Production"], remark: "电商管理后台", autoStart: true },
  { name: "kafka-broker", protocol: "tcp", localPort: 9092, remotePort: 9092, serverName: "Seoul Node", projectName: "Dev Environment", projectId: "p3", tags: ["API"], remark: "Kafka 消息中间件", autoStart: false },
  { name: "minio-storage", protocol: "http", localPort: 9001, remotePort: 8088, serverName: "Frankfurt Relay", projectName: "Dev Environment", projectId: "p3", tags: ["Personal"], remark: "MinIO 对象存储", autoStart: false },
  { name: "vault-secrets", protocol: "tcp", localPort: 8200, remotePort: 8200, serverName: "Frankfurt Relay", projectName: "SSH Bastion", projectId: "p8", tags: ["SSH", "Production"], remark: "Vault 密钥管理", autoStart: true },
  { name: "jenkins-ci", protocol: "http", localPort: 8081, remotePort: 8089, serverName: "Singapore Hub", projectName: "Dev Environment", projectId: "p3", tags: ["Demo"], remark: "Jenkins CI 流水线", autoStart: false },
  { name: "portainer", protocol: "http", localPort: 9443, remotePort: 8090, serverName: "Seoul Node", projectName: "Dev Environment", projectId: "p3", tags: ["Demo"], remark: "Portainer 容器管理", autoStart: false },
  { name: "docs-site", protocol: "http", localPort: 1313, remotePort: 8091, serverName: "US-West Gateway", projectName: "Web App Frontend", projectId: "p2", tags: ["Frontend", "Personal"], remark: "Hugo 文档站点", autoStart: false },
  { name: "ftp-server", protocol: "tcp", localPort: 21, remotePort: 2121, serverName: "US-West Gateway", projectName: "Dev Environment", projectId: "p3", tags: ["Personal"], remark: "FTP 文件传输", autoStart: false },
  { name: "auth-service", protocol: "http", localPort: 6000, remotePort: 8092, serverName: "Tokyo Edge", projectName: "My API Service", projectId: "p1", tags: ["API", "Production"], remark: "OAuth2 认证服务", autoStart: true },
  { name: "chat-socket", protocol: "tcp", localPort: 4500, remotePort: 4500, serverName: "Singapore Hub", projectName: "Realtime Chat", projectId: "p9", tags: ["API", "Staging"], remark: "Socket.io 长连接", autoStart: false },
  { name: "dns-server", protocol: "tcp", localPort: 53, remotePort: 5353, serverName: "Seoul Node", projectName: "IoT Gateway", projectId: "p5", tags: ["API"], remark: "DNS 解析服务", autoStart: false },
]

/* ── 构造完整 Tunnel 数组 ── */
export const mockTunnels: Tunnel[] = tunnelSeeds.map((seed, i) => {
  const status = statusPool[i % statusPool.length]
  const serverDomain = serverDomains[seed.serverName] ?? "gate.dev"
  const publicAddr = `${serverDomain}:${seed.remotePort}`
  const created = now - (30 - i) * day - rand(0, 23) * hour
  const updated = now - rand(0, 30) * min

  const running = status === "running"
  const baseSpeed = running ? rand(2, 256) * 1024 : 0
  const upSpeed = running ? Math.floor(baseSpeed * (0.3 + Math.random() * 0.5)) : 0
  const downSpeed = running ? baseSpeed : 0

  const totalUp = running ? rand(10, 5000) * 1024 * 1024 : rand(0, 5) * 1024 * 1024
  const totalDown = running ? rand(20, 12000) * 1024 * 1024 : rand(0, 10) * 1024 * 1024

  const connectionCount = running ? rand(1, 24) : 0
  const uptime = running ? rand(60, 3600 * 24 * 3) : 0

  return {
    id: `t${i + 1}`,
    name: seed.name,
    protocol: seed.protocol,
    localHost: "127.0.0.1",
    localPort: seed.localPort,
    remotePort: seed.remotePort,
    publicAddr,
    remark: seed.remark,
    status,
    autoStart: seed.autoStart,
    compression: false,
    encryption: false,
    tags: seed.tags,
    serverName: seed.serverName,
    projectName: seed.projectName,
    projectId: seed.projectId,
    pinned: i < 3, // 前三个固定
    favorite: [0, 2, 5, 8, 11, 18].includes(i), // 部分收藏
    traffic: {
      uploadSpeed: upSpeed,
      downloadSpeed: downSpeed,
      totalUpload: totalUp,
      totalDownload: totalDown,
      todayUpload: Math.floor(totalUp * (0.05 + Math.random() * 0.15)),
      todayDownload: Math.floor(totalDown * (0.05 + Math.random() * 0.15)),
      history: makeHistory(baseSpeed),
    },
    statistics: {
      uptime,
      connections: connectionCount,
      totalConnections: rand(100, 50000),
      requests: rand(1000, 999999),
      avgLatency: running ? rand(5, 320) : 0,
      peakSpeed: Math.floor(baseSpeed * (2 + Math.random() * 3)),
    },
    connections: running ? makeConnections(connectionCount, seed.protocol) : [],
    logs: makeLogs(rand(12, 40)),
    lastStartedAt: running
      ? pick(["刚刚", `${rand(1, 9)} 分钟前`, `${rand(1, 5)} 小时前`])
      : status === "stopped"
        ? "—"
        : pick([`${rand(1, 3)} 小时前`, `${rand(1, 2)} 天前`]),
    createdAt: new Date(created).toISOString(),
    updatedAt: new Date(updated).toISOString(),
  }
})

/* ── 默认表单值 ── */
export const defaultTunnelForm = {
  name: "",
  protocol: "http" as TunnelProtocol,
  localHost: "127.0.0.1",
  localPort: null as number | null,
  remotePort: null as number | null,
  projectId: mockProjects[0].id,
  serverName: mockServerNames[0],
  autoStart: false,
  remark: "",
  tags: [] as string[],
}

export { mockProjects as mockTunnelProjects }
