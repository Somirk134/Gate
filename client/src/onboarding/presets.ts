import type { TunnelProtocol } from "@/views/tunnels/types"

export type PresetAvailability = "available" | "reserved"

export interface TunnelTemplatePreset {
  id: string
  title: string
  description: string
  icon: string
  protocol: TunnelProtocol
  suggestedName: string
  localPort: number
  remotePort: number
  tags: string[]
  availability: PresetAvailability
}

export interface QuickStartScenario {
  id: string
  title: string
  description: string
  icon: string
  templateId: string
  suggestedName: string
  localPort: number
  remotePort: number
  tags: string[]
}

export const tunnelTemplates: TunnelTemplatePreset[] = [
  {
    id: "http-reserved",
    title: "HTTP",
    description: "预留模板，当前 Sprint 不新增 HTTP 协议能力。",
    icon: "globe",
    protocol: "http",
    suggestedName: "http-preview",
    localPort: 3000,
    remotePort: 18080,
    tags: ["Reserved"],
    availability: "reserved",
  },
  {
    id: "tcp",
    title: "TCP",
    description: "通用 TCP 服务，适合先打通端口映射和连通性。",
    icon: "router",
    protocol: "tcp",
    suggestedName: "tcp-service",
    localPort: 8080,
    remotePort: 18080,
    tags: ["TCP"],
    availability: "available",
  },
  {
    id: "webhook",
    title: "Webhook",
    description: "用于本地回调调试，保持现有隧道协议不变。",
    icon: "link",
    protocol: "tcp",
    suggestedName: "webhook-dev",
    localPort: 3000,
    remotePort: 18081,
    tags: ["Webhook", "Dev"],
    availability: "available",
  },
  {
    id: "minecraft",
    title: "Minecraft",
    description: "Minecraft Java Server 默认端口。",
    icon: "box",
    protocol: "tcp",
    suggestedName: "minecraft-server",
    localPort: 25565,
    remotePort: 25565,
    tags: ["Game"],
    availability: "available",
  },
  {
    id: "ssh",
    title: "SSH",
    description: "远程 SSH 调试入口，建议使用高位公网端口。",
    icon: "terminal",
    protocol: "tcp",
    suggestedName: "ssh-dev",
    localPort: 22,
    remotePort: 2222,
    tags: ["SSH"],
    availability: "available",
  },
  {
    id: "mysql",
    title: "MySQL",
    description: "本地 MySQL 调试或临时访问。",
    icon: "database",
    protocol: "tcp",
    suggestedName: "mysql-dev",
    localPort: 3306,
    remotePort: 3306,
    tags: ["Database"],
    availability: "available",
  },
  {
    id: "redis",
    title: "Redis",
    description: "Redis 默认端口，建议只在可信网络下使用。",
    icon: "database",
    protocol: "tcp",
    suggestedName: "redis-cache",
    localPort: 6379,
    remotePort: 6379,
    tags: ["Database", "Cache"],
    availability: "available",
  },
  {
    id: "postgresql",
    title: "PostgreSQL",
    description: "PostgreSQL 默认端口。",
    icon: "database",
    protocol: "tcp",
    suggestedName: "postgres-dev",
    localPort: 5432,
    remotePort: 5432,
    tags: ["Database"],
    availability: "available",
  },
  {
    id: "rdp",
    title: "RDP",
    description: "Windows 远程桌面默认端口，建议配合强密码和访问控制。",
    icon: "monitor",
    protocol: "tcp",
    suggestedName: "windows-rdp",
    localPort: 3389,
    remotePort: 13389,
    tags: ["Remote"],
    availability: "available",
  },
]

export const quickStartScenarios: QuickStartScenario[] = [
  {
    id: "local-dev",
    title: "本地开发",
    description: "Vite、Next、Express 等本地服务调试。",
    icon: "code",
    templateId: "tcp",
    suggestedName: "local-dev",
    localPort: 3000,
    remotePort: 18080,
    tags: ["Dev"],
  },
  {
    id: "payment-callback",
    title: "支付回调",
    description: "支付宝、微信支付、Stripe 回调本地联调。",
    icon: "credit-card",
    templateId: "webhook",
    suggestedName: "payment-callback",
    localPort: 3000,
    remotePort: 18082,
    tags: ["Webhook", "Payment"],
  },
  {
    id: "webhook",
    title: "Webhook",
    description: "GitHub、飞书、企业微信等回调调试。",
    icon: "link",
    templateId: "webhook",
    suggestedName: "webhook-dev",
    localPort: 3000,
    remotePort: 18081,
    tags: ["Webhook"],
  },
  {
    id: "ssh",
    title: "SSH",
    description: "把本机 SSH 暂时暴露给可信设备。",
    icon: "terminal",
    templateId: "ssh",
    suggestedName: "ssh-dev",
    localPort: 22,
    remotePort: 2222,
    tags: ["SSH"],
  },
  {
    id: "database",
    title: "数据库",
    description: "MySQL、PostgreSQL、Redis 等本地数据库调试。",
    icon: "database",
    templateId: "mysql",
    suggestedName: "mysql-dev",
    localPort: 3306,
    remotePort: 3306,
    tags: ["Database"],
  },
  {
    id: "nas",
    title: "NAS",
    description: "NAS Web 管理或文件服务临时访问。",
    icon: "hard-drive",
    templateId: "tcp",
    suggestedName: "nas-access",
    localPort: 5000,
    remotePort: 15000,
    tags: ["NAS"],
  },
  {
    id: "docker",
    title: "Docker",
    description: "容器内服务、Portainer 或本地 Compose 服务。",
    icon: "boxes",
    templateId: "tcp",
    suggestedName: "docker-service",
    localPort: 8080,
    remotePort: 18083,
    tags: ["Docker"],
  },
]

export function findTemplate(id: string): TunnelTemplatePreset {
  return tunnelTemplates.find((template) => template.id === id) ?? tunnelTemplates[1]
}
