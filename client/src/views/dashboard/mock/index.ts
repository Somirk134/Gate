/* ==================================================================
   Dashboard Mock 数据
   ------------------------------------------------------------------
   统一 Mock 数据源。所有 Dashboard 组件不写死数据，统一从此引入。
   后续替换为真实接口时，只需将 store 中的 load 动作改为 API 调用，
   并保持返回类型与 types/index.ts 一致即可。
   ================================================================== */

import type {
  DashboardProject,
  DashboardTunnel,
  DashboardServer,
  DashboardActivity,
  DashboardStatistics,
  DashboardResource,
  DashboardNews,
  DashboardQuickAction,
} from "../types"

const now = Date.now()
const min = 60 * 1000
const hour = 60 * min
const day = 24 * hour

/* ── 开发者语录 ── */
export const developerQuotes: string[] = [
  "代码是写给人看的，顺便能让机器执行。",
  "简单是可靠的先决条件。",
  "过早优化是万恶之源。",
  "先让它工作，再让它正确，最后让它快速。",
  "最好的错误信息是那些不会出现的错误。",
  "Talk is cheap. Show me the code.",
  "任何傻瓜都能写出计算机能理解的代码，好的程序员能写出人能理解的代码。",
  "软件就像洋葱，剥开一层还有一层。",
  "调试代码比写代码难一倍，所以如果你尽全力写代码，按定义你没有能力调试它。",
  "好的架构让重量级的事情变得轻盈。",
]

/* ── 当前版本 ── */
export const appVersion = "v0.4.0-beta"

/* ── 快捷操作 ── */
export const quickActions: DashboardQuickAction[] = [
  { id: "new-project", icon: "plus", label: "新建项目", shortcut: "Ctrl N", variant: "primary" },
  { id: "connect-server", icon: "plug", label: "连接服务器", shortcut: "Ctrl K", variant: "info" },
  { id: "new-tunnel", icon: "link", label: "新建 Tunnel", shortcut: "Ctrl T", variant: "success" },
  { id: "open-logs", icon: "scroll-text", label: "打开日志", shortcut: "Ctrl L", variant: "warning" },
  { id: "open-settings", icon: "settings", label: "打开设置", shortcut: "Ctrl ,", variant: "primary" },
  { id: "check-update", icon: "refresh", label: "检查更新", shortcut: "Ctrl U", variant: "info" },
]

/* ── 项目（不少于 6 个） ── */
export const mockProjects: DashboardProject[] = [
  {
    id: "p1",
    name: "My API Service",
    icon: "package",
    description: "后端微服务集群，包含用户、订单、支付三大模块",
    tunnelCount: 8,
    runningCount: 3,
    lastStartedAt: "2 小时前",
    pinned: true,
    favorite: true,
    status: "online",
  },
  {
    id: "p2",
    name: "Web App Frontend",
    icon: "globe",
    description: "React + Vite 前端应用，本地开发热更新",
    tunnelCount: 3,
    runningCount: 1,
    lastStartedAt: "1 天前",
    pinned: true,
    favorite: false,
    status: "online",
  },
  {
    id: "p3",
    name: "Dev Environment",
    icon: "code",
    description: "完整开发环境，数据库 + 缓存 + 队列",
    tunnelCount: 5,
    runningCount: 5,
    lastStartedAt: "刚刚",
    pinned: false,
    favorite: true,
    status: "online",
  },
  {
    id: "p4",
    name: "Monitoring Stack",
    icon: "activity",
    description: "Prometheus + Grafana 监控可视化平台",
    tunnelCount: 4,
    runningCount: 2,
    lastStartedAt: "3 小时前",
    pinned: false,
    favorite: false,
    status: "connecting",
  },
  {
    id: "p5",
    name: "ML Training Server",
    icon: "cpu",
    description: "GPU 训练任务远程调试与可视化",
    tunnelCount: 2,
    runningCount: 0,
    lastStartedAt: "2 天前",
    pinned: false,
    favorite: false,
    status: "offline",
  },
  {
    id: "p6",
    name: "IoT Gateway",
    icon: "router",
    description: "物联网设备网关，MQTT 协议转发",
    tunnelCount: 6,
    runningCount: 4,
    lastStartedAt: "5 小时前",
    pinned: false,
    favorite: true,
    status: "online",
  },
]

/* ── 运行中的隧道 ── */
export const mockTunnels: DashboardTunnel[] = [
  {
    id: "t1",
    name: "api-gateway",
    protocol: "https",
    status: "online",
    localPort: 8080,
    publicPort: 30443,
    publicHost: "gate.dev",
    uploadSpeed: 128.5,
    downloadSpeed: 642.3,
    connections: 12,
    projectId: "p1",
  },
  {
    id: "t2",
    name: "web-frontend",
    protocol: "http",
    status: "online",
    localPort: 5173,
    publicPort: 30080,
    publicHost: "gate.dev",
    uploadSpeed: 24.1,
    downloadSpeed: 88.7,
    connections: 3,
    projectId: "p2",
  },
  {
    id: "t3",
    name: "postgres-db",
    protocol: "tcp",
    status: "online",
    localPort: 5432,
    publicPort: 35432,
    publicHost: "gate.dev",
    uploadSpeed: 512.8,
    downloadSpeed: 1024.4,
    connections: 5,
    projectId: "p3",
  },
  {
    id: "t4",
    name: "redis-cache",
    protocol: "tcp",
    status: "online",
    localPort: 6379,
    publicPort: 36379,
    publicHost: "gate.dev",
    uploadSpeed: 8.2,
    downloadSpeed: 15.6,
    connections: 8,
    projectId: "p3",
  },
  {
    id: "t5",
    name: "grafana-ui",
    protocol: "https",
    status: "connecting",
    localPort: 3000,
    publicPort: 33000,
    publicHost: "gate.dev",
    uploadSpeed: 0,
    downloadSpeed: 0,
    connections: 0,
    projectId: "p4",
  },
]

/* ── 服务器 ── */
export const mockServers: DashboardServer[] = [
  {
    id: "s1",
    name: "Tokyo Edge",
    region: "东京 / 亚太",
    ip: "13.112.84.20",
    version: "gate-server 0.4.0",
    ping: 42,
    cpu: 38,
    memory: 64,
    disk: 51,
    network: 22,
    status: "online",
    connected: true,
  },
  {
    id: "s2",
    name: "Frankfurt Relay",
    region: "法兰克福 / 欧洲",
    ip: "18.184.107.91",
    version: "gate-server 0.3.9",
    ping: 168,
    cpu: 71,
    memory: 82,
    disk: 67,
    network: 55,
    status: "online",
    connected: true,
  },
  {
    id: "s3",
    name: "US-West Backup",
    region: "洛杉矶 / 北美",
    ip: "54.215.42.108",
    version: "gate-server 0.4.0",
    ping: 0,
    cpu: 0,
    memory: 0,
    disk: 0,
    network: 0,
    status: "connecting",
    connected: false,
  },
]

/* ── 活动 ── */
export const mockActivities: DashboardActivity[] = [
  {
    id: "a1",
    type: "start",
    title: "启动隧道 api-gateway",
    description: "本地 8080 → gate.dev:30443 (HTTPS)",
    timestamp: now - 5 * min,
  },
  {
    id: "a2",
    type: "connect",
    title: "连接服务器 Tokyo Edge",
    description: "延迟 42ms · 版本 0.4.0",
    timestamp: now - 18 * min,
  },
  {
    id: "a3",
    type: "config",
    title: "修改项目配置 Dev Environment",
    description: "更新了 redis-cache 隧道的远程端口",
    timestamp: now - 52 * min,
  },
  {
    id: "a4",
    type: "create",
    title: "创建项目 IoT Gateway",
    description: "包含 6 条隧道配置",
    timestamp: now - 3 * hour,
  },
  {
    id: "a5",
    type: "stop",
    title: "停止隧道 ml-tensorboard",
    description: "用户手动停止",
    timestamp: now - 5 * hour,
  },
  {
    id: "a6",
    type: "update",
    title: "Gate 客户端检查更新",
    description: "当前已是最新版本 v0.4.0-beta",
    timestamp: now - 8 * hour,
  },
  {
    id: "a7",
    type: "delete",
    title: "删除隧道旧 webhook-test",
    description: "清理无用配置",
    timestamp: now - day - 2 * hour,
  },
]

/* ── 统计 ── */
export const mockStatistics: DashboardStatistics = {
  projectCount: 6,
  tunnelCount: 28,
  runningTunnel: 15,
  todayUpload: 2.4 * 1024 * 1024 * 1024,
  todayDownload: 8.7 * 1024 * 1024 * 1024,
  onlineTime: 6 * hour + 42 * min,
}

/* ── 资源监控 ── */
export const mockResource: DashboardResource = {
  cpu: 34,
  memory: 58,
  traffic: 41,
  connection: 67,
}

/* ── 资讯 ── */
export const mockNews: DashboardNews[] = [
  {
    id: "n1",
    type: "release",
    title: "Gate v0.4.0-beta 发布",
    version: "v0.4.0-beta",
    date: "2 天前",
    summary: "新增隧道流量监控、服务器多区域支持、性能与稳定性优化。",
    url: "#",
  },
  {
    id: "n2",
    type: "github",
    title: "Star 数突破 1.2k",
    date: "5 天前",
    summary: "感谢社区支持！Gate 已在 GitHub 获得 1200+ Star。",
    url: "#",
  },
  {
    id: "n3",
    type: "changelog",
    title: "修复 HTTPS 隧道偶发断连",
    date: "1 周前",
    summary: "解决长连接空闲超时未正确发送心跳的问题。",
    url: "#",
  },
]
