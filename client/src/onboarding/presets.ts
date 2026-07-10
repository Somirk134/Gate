import type { TunnelProtocol } from '@/views/tunnels/types'

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
    id: 'http',
    title: 'HTTP',
    description: 'HTTP/1.1 service with host and path routing.',
    icon: 'globe',
    protocol: 'http',
    suggestedName: 'http-service',
    localPort: 3000,
    remotePort: 18080,
    tags: ['HTTP'],
  },
  {
    id: 'tcp',
    title: 'TCP',
    description: 'General TCP service for port mapping and connectivity checks.',
    icon: 'router',
    protocol: 'tcp',
    suggestedName: 'tcp-service',
    localPort: 8080,
    remotePort: 18080,
    tags: ['TCP'],
  },
  {
    id: 'webhook',
    title: 'Webhook',
    description: 'Local webhook callback debugging entrypoint.',
    icon: 'link',
    protocol: 'http',
    suggestedName: 'webhook-dev',
    localPort: 3000,
    remotePort: 18081,
    tags: ['Webhook', 'Dev'],
  },
  {
    id: 'minecraft',
    title: 'Minecraft',
    description: 'Default port for Minecraft Java server.',
    icon: 'box',
    protocol: 'tcp',
    suggestedName: 'minecraft-server',
    localPort: 25565,
    remotePort: 25565,
    tags: ['Game'],
  },
  {
    id: 'ssh',
    title: 'SSH',
    description: 'Remote SSH debugging entrypoint; a high public port is recommended.',
    icon: 'terminal',
    protocol: 'tcp',
    suggestedName: 'ssh-dev',
    localPort: 22,
    remotePort: 2222,
    tags: ['SSH'],
  },
  {
    id: 'mysql',
    title: 'MySQL',
    description: 'Local MySQL debugging or temporary access.',
    icon: 'database',
    protocol: 'tcp',
    suggestedName: 'mysql-dev',
    localPort: 3306,
    remotePort: 3306,
    tags: ['Database'],
  },
  {
    id: 'redis',
    title: 'Redis',
    description: 'Default Redis port; use only on trusted networks.',
    icon: 'database',
    protocol: 'tcp',
    suggestedName: 'redis-cache',
    localPort: 6379,
    remotePort: 6379,
    tags: ['Database', 'Cache'],
  },
  {
    id: 'postgresql',
    title: 'PostgreSQL',
    description: 'Default PostgreSQL port.',
    icon: 'database',
    protocol: 'tcp',
    suggestedName: 'postgres-dev',
    localPort: 5432,
    remotePort: 5432,
    tags: ['Database'],
  },
  {
    id: 'rdp',
    title: 'RDP',
    description: 'Default Windows Remote Desktop port; strong passwords and access control are recommended.',
    icon: 'monitor',
    protocol: 'tcp',
    suggestedName: 'windows-rdp',
    localPort: 3389,
    remotePort: 13389,
    tags: ['Remote'],
  },
]

export const quickStartScenarios: QuickStartScenario[] = [
  {
    id: 'local-dev',
    title: 'Local Dev',
    description: 'Local service debugging for Vite, Next, Express, and similar tools.',
    icon: 'code',
    templateId: 'http',
    suggestedName: 'local-dev',
    localPort: 3000,
    remotePort: 18080,
    tags: ['Dev'],
  },
  {
    id: 'payment-callback',
    title: 'Payment Callback',
    description: 'Local callback debugging for Alipay, WeChat Pay, Stripe, and similar providers.',
    icon: 'credit-card',
    templateId: 'webhook',
    suggestedName: 'payment-callback',
    localPort: 3000,
    remotePort: 18082,
    tags: ['Webhook', 'Payment'],
  },
  {
    id: 'webhook',
    title: 'Webhook',
    description: 'Callback debugging for GitHub, Feishu, WeCom, and similar integrations.',
    icon: 'link',
    templateId: 'webhook',
    suggestedName: 'webhook-dev',
    localPort: 3000,
    remotePort: 18081,
    tags: ['Webhook'],
  },
  {
    id: 'ssh',
    title: 'SSH',
    description: 'Temporarily expose local SSH to trusted devices.',
    icon: 'terminal',
    templateId: 'ssh',
    suggestedName: 'ssh-dev',
    localPort: 22,
    remotePort: 2222,
    tags: ['SSH'],
  },
  {
    id: 'database',
    title: 'Database',
    description: 'Local database debugging for MySQL, PostgreSQL, Redis, and similar services.',
    icon: 'database',
    templateId: 'mysql',
    suggestedName: 'mysql-dev',
    localPort: 3306,
    remotePort: 3306,
    tags: ['Database'],
  },
  {
    id: 'nas',
    title: 'NAS',
    description: 'Temporary access to NAS web admin or file services.',
    icon: 'hard-drive',
    templateId: 'tcp',
    suggestedName: 'nas-access',
    localPort: 5000,
    remotePort: 15000,
    tags: ['NAS'],
  },
  {
    id: 'docker',
    title: 'Docker',
    description: 'Services inside containers, Portainer, or local Compose services.',
    icon: 'boxes',
    templateId: 'tcp',
    suggestedName: 'docker-service',
    localPort: 8080,
    remotePort: 18083,
    tags: ['Docker'],
  },
]

export function findTemplate(id: string): TunnelTemplatePreset {
  return tunnelTemplates.find((template) => template.id === id) ?? tunnelTemplates[1]
}
