/* ==================================================================
   Project Mock 数据
   ------------------------------------------------------------------
   统一 Mock 数据源。所有 Project 组件不写死数据，统一从此引入。
   生成 20 个不同颜色/图标/状态/隧道数/标签/服务器的项目。
   后续替换为真实接口时，只需将 store 的 load 动作改为 API 调用，
   并保持返回类型与 types/index.ts 一致即可。
   ================================================================== */

import type { Project, MockTunnel } from "../types"

const now = Date.now()
const min = 60 * 1000
const hour = 60 * min
const day = 24 * hour

const servers = [
  "Tokyo Edge",
  "Frankfurt Relay",
  "US-West Gateway",
  "Singapore Hub",
  "Seoul Node",
]

const iconPool = [
  "package", "globe", "database", "servers", "cloud", "code", "box", "terminal",
  "activity", "cpu", "router", "layers", "boxes", "network", "shield", "zap",
  "rocket", "hard-drive", "link", "plug",
]

const colorPool = [
  "blue", "green", "purple", "orange", "red", "cyan", "pink", "indigo",
  "teal", "amber", "slate",
] as const

const projectSeeds: Array<Pick<Project, "name" | "description" | "icon" | "color" | "tags" | "serverName" | "tunnelCount" | "runningTunnelCount" | "status" | "autoStart" | "remark">> = [
  { name: "My API Service", description: "后端微服务集群，包含用户、订单、支付三大模块", icon: "package", color: "blue", tags: ["Work", "Production"], serverName: "Tokyo Edge", tunnelCount: 8, runningTunnelCount: 5, status: "running", autoStart: true, remark: "核心生产服务，请勿随意重启" },
  { name: "Web App Frontend", description: "React + Vite 前端应用，本地开发热更新", icon: "globe", color: "cyan", tags: ["Work", "Staging"], serverName: "Tokyo Edge", tunnelCount: 3, runningTunnelCount: 1, status: "partial", autoStart: false, remark: "" },
  { name: "Dev Environment", description: "完整开发环境，数据库 + 缓存 + 消息队列", icon: "code", color: "green", tags: ["Personal"], serverName: "Singapore Hub", tunnelCount: 5, runningTunnelCount: 5, status: "running", autoStart: true, remark: "日常开发主力" },
  { name: "Monitoring Stack", description: "Prometheus + Grafana + AlertManager 监控可视化平台", icon: "activity", color: "purple", tags: ["Work", "Production"], serverName: "Frankfurt Relay", tunnelCount: 4, runningTunnelCount: 2, status: "partial", autoStart: false, remark: "" },
  { name: "ML Training Server", description: "GPU 训练任务远程调试与 TensorBoard 可视化", icon: "cpu", color: "pink", tags: ["Client"], serverName: "US-West Gateway", tunnelCount: 2, runningTunnelCount: 0, status: "stopped", autoStart: false, remark: "按需启动" },
  { name: "IoT Gateway", description: "物联网设备网关，MQTT 协议转发与设备管理", icon: "router", color: "teal", tags: ["Client", "Production"], serverName: "Seoul Node", tunnelCount: 6, runningTunnelCount: 6, status: "running", autoStart: true, remark: "" },
  { name: "E-Commerce Platform", description: "SpringBoot + Vue + Redis + MySQL 商城系统", icon: "package", color: "orange", tags: ["Work", "Production"], serverName: "Tokyo Edge", tunnelCount: 10, runningTunnelCount: 7, status: "running", autoStart: true, remark: "电商核心系统" },
  { name: "Blog & CMS", description: "Ghost 博客与内容管理系统，支持多主题", icon: "globe", color: "indigo", tags: ["Open Source", "Personal"], serverName: "Frankfurt Relay", tunnelCount: 2, runningTunnelCount: 2, status: "running", autoStart: false, remark: "" },
  { name: "Game Server", description: "Minecraft 多人游戏服务器，支持 Mod 与白名单", icon: "box", color: "green", tags: ["Personal"], serverName: "US-West Gateway", tunnelCount: 1, runningTunnelCount: 0, status: "stopped", autoStart: false, remark: "周末开放" },
  { name: "CI/CD Pipeline", description: "Jenkins + GitLab Runner 持续集成与部署流水线", icon: "rocket", color: "amber", tags: ["Work", "Staging"], serverName: "Singapore Hub", tunnelCount: 4, runningTunnelCount: 3, status: "running", autoStart: true, remark: "" },
  { name: "Database Cluster", description: "PostgreSQL 主从 + Redis 哨兵 + MongoDB 分片", icon: "database", color: "red", tags: ["Work", "Production"], serverName: "Tokyo Edge", tunnelCount: 7, runningTunnelCount: 7, status: "running", autoStart: true, remark: "数据敏感，谨慎操作" },
  { name: "SSH Bastion", description: "SSH 跳板机与密钥管理，支持审计日志", icon: "shield", color: "slate", tags: ["Work"], serverName: "Frankfurt Relay", tunnelCount: 3, runningTunnelCount: 3, status: "running", autoStart: true, remark: "" },
  { name: "Message Queue", description: "RabbitMQ + Kafka 消息中间件集群", icon: "layers", color: "blue", tags: ["Work", "Production"], serverName: "Seoul Node", tunnelCount: 5, runningTunnelCount: 0, status: "stopped", autoStart: false, remark: "" },
  { name: "Static Site", description: "Hugo 生成的静态文档站点，CDN 加速", icon: "box", color: "cyan", tags: ["Open Source"], serverName: "US-West Gateway", tunnelCount: 1, runningTunnelCount: 1, status: "running", autoStart: false, remark: "" },
  { name: "Realtime Chat", description: "WebSocket 即时通讯服务，支持群聊与文件传输", icon: "zap", color: "purple", tags: ["Client", "Staging"], serverName: "Singapore Hub", tunnelCount: 4, runningTunnelCount: 2, status: "partial", autoStart: false, remark: "" },
  { name: "Video Streaming", description: "FFmpeg 推流 + Nginx-RTMP 直播分发", icon: "activity", color: "pink", tags: ["Client"], serverName: "Tokyo Edge", tunnelCount: 3, runningTunnelCount: 0, status: "stopped", autoStart: false, remark: "活动时启动" },
  { name: "File Storage", description: "MinIO 对象存储与 WebDAV 文件共享", icon: "hard-drive", color: "teal", tags: ["Personal"], serverName: "Frankfurt Relay", tunnelCount: 2, runningTunnelCount: 2, status: "running", autoStart: false, remark: "" },
  { name: "DevOps Toolkit", description: "Portainer + cAdvisor + Traefik 容器管理套件", icon: "terminal", color: "orange", tags: ["Work"], serverName: "Seoul Node", tunnelCount: 6, runningTunnelCount: 4, status: "partial", autoStart: true, remark: "" },
  { name: "API Gateway", description: "Kong 网关与限流熔断，支持插件扩展", icon: "network", color: "indigo", tags: ["Work", "Production"], serverName: "Tokyo Edge", tunnelCount: 5, runningTunnelCount: 5, status: "running", autoStart: true, remark: "" },
  { name: "Demo Showcase", description: "对外演示项目集合，支持一键切换环境", icon: "rocket", color: "amber", tags: ["Demo", "Client"], serverName: "Singapore Hub", tunnelCount: 8, runningTunnelCount: 0, status: "stopped", autoStart: false, remark: "客户演示用" },
]

/* 生成统计数据的辅助函数 */
function makeStats(tunnelCount: number, running: number, baseTraffic: number) {
  return {
    todayTraffic: Math.floor(baseTraffic * (0.5 + Math.random())),
    totalTraffic: Math.floor(baseTraffic * (20 + Math.random() * 80)),
    uptime: Math.floor((running > 0 ? 1 : 0) * (hour * 6 + Math.random() * hour * 12)),
    connections: running * Math.floor(2 + Math.random() * 10),
    tunnelCount,
    runningTunnelCount: running,
  }
}

/* 构造完整 Project 数组 */
export const mockProjects: Project[] = projectSeeds.map((seed, i) => {
  const created = now - (20 - i) * day - Math.floor(Math.random() * hour)
  const updated = now - Math.floor(Math.random() * day)
  return {
    id: `p${i + 1}`,
    name: seed.name,
    description: seed.description,
    icon: seed.icon,
    color: seed.color,
    tags: seed.tags,
    serverName: seed.serverName,
    autoStart: seed.autoStart,
    remark: seed.remark,
    status: seed.status,
    pinned: i < 2, // 前两个固定
    favorite: [0, 2, 5, 6, 10].includes(i), // 部分收藏
    lastUsedAt: updated,
    tunnelCount: seed.tunnelCount,
    runningTunnelCount: seed.runningTunnelCount,
    statistics: makeStats(seed.tunnelCount, seed.runningTunnelCount, 1024 * 1024 * 1024),
    lastStartedAt:
      seed.status === "stopped"
        ? "—"
        : seed.runningTunnelCount > 0
          ? "刚刚"
          : `${Math.floor(Math.random() * 5) + 1} 小时前`,
    createdAt: new Date(created).toISOString(),
    updatedAt: new Date(updated).toISOString(),
  }
})

/* ── 详情页 Mock Tunnel（每个项目复用同一组示例） ── */
export const mockTunnels: MockTunnel[] = [
  { id: "t1", name: "api-gateway", protocol: "https", localAddr: "localhost:3000", remoteAddr: ":8080", publicAddr: "gate.dev:30443", status: "online", downSpeed: "24 KB/s", upSpeed: "12 KB/s", connections: 8 },
  { id: "t2", name: "web-frontend", protocol: "http", localAddr: "localhost:5173", remoteAddr: ":80", publicAddr: "gate.dev:30080", status: "online", downSpeed: "8 KB/s", upSpeed: "3 KB/s", connections: 3 },
  { id: "t3", name: "postgres-db", protocol: "tcp", localAddr: "localhost:5432", remoteAddr: ":5432", publicAddr: "gate.dev:35432", status: "offline", downSpeed: "--", upSpeed: "--", connections: 0 },
  { id: "t4", name: "redis-cache", protocol: "tcp", localAddr: "localhost:6379", remoteAddr: ":6379", publicAddr: "gate.dev:36379", status: "starting", downSpeed: "--", upSpeed: "--", connections: 0 },
  { id: "t5", name: "admin-panel", protocol: "http", localAddr: "localhost:3001", remoteAddr: ":3000", publicAddr: "gate.dev:33001", status: "error", downSpeed: "--", upSpeed: "--", connections: 0 },
  { id: "t6", name: "ssh-tunnel", protocol: "tcp", localAddr: "localhost:22", remoteAddr: ":2222", publicAddr: "gate.dev:32222", status: "online", downSpeed: "1 KB/s", upSpeed: "1 KB/s", connections: 1 },
]

/* ── 默认表单值 ── */
export const defaultProjectForm = {
  name: "",
  icon: "package",
  color: "blue" as const,
  description: "",
  serverName: servers[0],
  autoStart: false,
  tags: [] as string[],
  remark: "",
}

export const mockServerNames = servers
export const mockIconPool = iconPool
export const mockColorPool = colorPool
