/* ==================================================================
   Server Mock — 主数据源
   ------------------------------------------------------------------
   统一 Mock 数据源。所有 Server 组件不写死数据，统一从此引入。
   生成 10 台服务器：不同地区 / 状态 / CPU / Memory / Tunnel 数 /
   Project 数 / Traffic / Version / Health。

   后续替换为真实 Rust Server 时，只需将 store 的 load 动作改为
   API 调用，并保持返回类型与 types/index.ts 一致即可无缝迁移。
   ================================================================== */

import type {
  Server,
  ServerKind,
  ServerOverviewInfo,
  ServerStatus,
} from "../types"
import { makeTraffic } from "./traffic"
import { makeMonitor } from "./monitor"
import { makeStatistics } from "./statistics"
import { makeLogs, makeConnections } from "./logs"
import { buildHealth } from "./health"

const now = Date.now()
const min = 60 * 1000
const hour = 60 * min
const day = 24 * hour

function rand(min: number, max: number): number {
  return Math.floor(Math.random() * (max - min + 1)) + min
}

function pick<T>(arr: T[]): T {
  return arr[rand(0, arr.length - 1)]
}

/* ── 概览信息种子 ── */
const osPool = [
  "Ubuntu 22.04 LTS",
  "Debian 12",
  "CentOS Stream 9",
  "Alpine Linux 3.19",
  "macOS 14.4",
  "Windows Server 2022",
  "Fedora 39",
  "Arch Linux",
  "Rocky Linux 9.3",
  "OpenSUSE Tumbleweed",
]

const archPool = ["x86_64", "x86_64", "x86_64", "aarch64", "aarch64"]

/* ── 生成概览 ── */
function makeOverview(version: string, running: boolean): ServerOverviewInfo {
  const install = now - rand(10, 300) * day
  return {
    hostname: `gate-${rand(10, 99)}.local`,
    os: pick(osPool),
    arch: pick(archPool),
    rustVersion: pick(["1.75.0", "1.76.0", "1.77.0", "1.78.0", "1.79.0"]),
    serverVersion: version,
    installTime: new Date(install).toISOString(),
    lastOnline: running ? "刚刚" : pick([`${rand(1, 9)} 分钟前`, `${rand(1, 5)} 小时前`]),
    lastHeartbeat: running ? "刚刚" : pick([`${rand(1, 3)} 小时前`, `${rand(1, 2)} 天前`]),
  }
}

/* ── 状态分布 ── */
const statusPool: ServerStatus[] = [
  "connected", "connected", "connected", "connected", "connected",
  "connecting", "reconnecting",
  "disconnected", "offline",
  "error",
]

/* ── 服务器种子（10 台） ── */
const serverSeeds: Array<{
  name: string
  kind: ServerKind
  region: string
  publicIp: string
  version: string
  tunnelCount: number
  projectCount: number
  tags: string[]
  remark: string
}> = [
  {
    name: "Tokyo Edge",
    kind: "cloud",
    region: "Tokyo, JP",
    publicIp: "54.183.45.12",
    version: "v0.2.1",
    tunnelCount: 8,
    projectCount: 4,
    tags: ["Production", "Cloud"],
    remark: "东京边缘节点，承载亚太流量",
  },
  {
    name: "Frankfurt Relay",
    kind: "cloud",
    region: "Frankfurt, DE",
    publicIp: "18.197.32.8",
    version: "v0.2.1",
    tunnelCount: 6,
    projectCount: 3,
    tags: ["Production", "Cloud"],
    remark: "法兰克福中继，欧洲入口",
  },
  {
    name: "US-West Gateway",
    kind: "cloud",
    region: "San Jose, US",
    publicIp: "13.229.45.10",
    version: "v0.2.0",
    tunnelCount: 5,
    projectCount: 2,
    tags: ["Production", "AWS"],
    remark: "美西网关，AWS us-west-1",
  },
  {
    name: "Singapore Hub",
    kind: "cloud",
    region: "Singapore, SG",
    publicIp: "52.74.211.88",
    version: "v0.2.1",
    tunnelCount: 7,
    projectCount: 3,
    tags: ["Production", "Cloud"],
    remark: "新加坡枢纽，东南亚分发",
  },
  {
    name: "Home NAS",
    kind: "nas",
    region: "Shanghai, CN",
    publicIp: "192.168.1.100",
    version: "v0.2.1",
    tunnelCount: 3,
    projectCount: 2,
    tags: ["Home", "Development"],
    remark: "群晖 NAS，家用内网穿透",
  },
  {
    name: "Dev Sandbox",
    kind: "docker",
    region: "Local, CN",
    publicIp: "172.17.0.2",
    version: "v0.2.1",
    tunnelCount: 2,
    projectCount: 1,
    tags: ["Development", "Testing"],
    remark: "本地 Docker 容器，开发测试",
  },
  {
    name: "Beijing Office",
    kind: "company",
    region: "Beijing, CN",
    publicIp: "10.0.12.34",
    version: "v0.2.0",
    tunnelCount: 4,
    projectCount: 2,
    tags: ["Testing"],
    remark: "公司北京机房测试服务器",
  },
  {
    name: "Seoul Node",
    kind: "personal",
    region: "Seoul, KR",
    publicIp: "121.78.55.220",
    version: "v0.2.1",
    tunnelCount: 3,
    projectCount: 2,
    tags: ["Production", "Cloud"],
    remark: "首尔个人 VPS",
  },
  {
    name: "Aliyun HK",
    kind: "cloud",
    region: "Hong Kong, HK",
    publicIp: "47.52.188.66",
    version: "v0.2.1",
    tunnelCount: 5,
    projectCount: 3,
    tags: ["Production", "Aliyun"],
    remark: "阿里云香港，低延迟节点",
  },
  {
    name: "Tencent GZ",
    kind: "cloud",
    region: "Guangzhou, CN",
    publicIp: "119.29.105.42",
    version: "v0.1.9",
    tunnelCount: 4,
    projectCount: 2,
    tags: ["Development", "Tencent"],
    remark: "腾讯云广州，华南备用",
  },
]

/* ── 构造完整 Server 数组 ── */
export const mockServers: Server[] = serverSeeds.map((seed, i) => {
  const status = statusPool[i % statusPool.length]
  const running = status === "connected"
  const created = now - (10 - i) * day - rand(0, 23) * hour
  const updated = now - rand(0, 30) * min

  const traffic = makeTraffic(running)
  const monitor = makeMonitor(running, traffic)
  const statistics = makeStatistics(running, seed.tunnelCount, seed.projectCount)
  const overview = makeOverview(seed.version, running)
  const health = buildHealth(running, seed.version)
  const connCount = running ? rand(1, 16) : 0

  return {
    id: `s${i + 1}`,
    name: seed.name,
    kind: seed.kind,
    region: seed.region,
    publicIp: seed.publicIp,
    version: seed.version,
    status,
    connectionMethod: "wss",
    ping: running ? rand(8, 220) : 0,
    tags: seed.tags,
    favorite: [0, 2, 4, 7].includes(i),
    recent: i < 6,
    overview,
    monitor,
    traffic,
    statistics,
    connections: running ? makeConnections(connCount) : [],
    logs: makeLogs(rand(15, 45)),
    health,
    settings: {
      name: seed.name,
      host: seed.publicIp,
      port: 7000,
      token: `gate_${Math.random().toString(36).slice(2, 14)}`,
      remark: seed.remark,
      heartbeatInterval: 30,
      reconnectInterval: 5,
      autoConnect: i < 5,
    },
    lastConnectedAt: running
      ? "刚刚"
      : pick([`${rand(1, 9)} 分钟前`, `${rand(1, 5)} 小时前`, `${rand(1, 2)} 天前`]),
    createdAt: new Date(created).toISOString(),
    updatedAt: new Date(updated).toISOString(),
  }
})

/* ── 默认表单值 ── */
export const defaultServerForm = {
  name: "",
  kind: "personal" as ServerKind,
  host: "",
  port: 7000 as number | null,
  token: "",
  region: "",
  remark: "",
  tags: [] as string[],
  heartbeatInterval: 30,
  reconnectInterval: 5,
  autoConnect: false,
}

/* ── 连接方式选项 ── */
export const connectionMethods = [
  { key: "wss", label: "WSS (安全 WebSocket)", description: "推荐，穿透性最好" },
  { key: "ws", label: "WS (WebSocket)", description: "明文，仅内网使用" },
  { key: "tcp", label: "TCP", description: "直连，低延迟" },
  { key: "grpc", label: "gRPC", description: "高性能 RPC" },
] as const
