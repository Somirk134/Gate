import type { TunnelFormData, TunnelProtocol } from '@/views/tunnels/types'

export type ServerOwnership = 'has-server' | 'no-server' | 'unknown-server'
export type DomainMode = 'has-domain' | 'no-domain' | 'skip-domain'

export type ServerEnvironmentId =
  | 'ubuntu'
  | 'debian'
  | 'centos'
  | 'windows-server'
  | 'docker'
  | 'bt-panel'
  | '1panel'
  | 'casaos'
  | 'unraid'
  | 'synology'

export interface KnowledgeCard {
  id: string
  title: string
  icon: string
  body: string
}

export interface CloudProviderOption {
  id: string
  name: string
  note: string
  tone: 'mainland' | 'global' | 'free' | 'experience'
}

export interface ServerEnvironmentOption {
  id: ServerEnvironmentId
  title: string
  icon: string
  description: string
  recommendedDeploy: string
  commandHint: string
  reserved?: boolean
}

export interface ScenarioRecommendation {
  id: string
  title: string
  description: string
  icon: string
  protocol: Extract<TunnelProtocol, 'tcp' | 'http' | 'https'>
  defaultName: string
  localHost: string
  localPort: number
  remotePort: number
  domainHint: string
  certificate: 'Auto' | 'Not required'
  tags: string[]
  reason: string
  domainStronglyRecommended?: boolean
}

export interface SmartWizardAnswers {
  serverOwnership: ServerOwnership | ''
  serverEnvironment: ServerEnvironmentId | ''
  serverAddress: string
  domainMode: DomainMode | ''
  domainName: string
  scenarioId: string
  customName: string
  customLocalPort: number | null
}

export interface SmartRecommendation {
  tunnelName: string
  server: string
  protocol: Extract<TunnelProtocol, 'tcp' | 'http' | 'https'>
  local: string
  remote: string
  domain: string
  certificate: string
  accessPreview: string
  deployMethod: string
  reasonList: string[]
  tags: string[]
  form: TunnelFormData
}

export const smartOnboardingKeys = {
  completed: 'gate.smartOnboarding.completed',
  neverShow: 'gate.smartOnboarding.neverShow',
  draft: 'gate.smartOnboarding.draft',
  oldCompleted: 'gate.firstLaunch.completed',
}

export const knowledgeCards: KnowledgeCard[] = [
  {
    id: 'tunnel',
    title: '什么是隧道？',
    icon: 'router',
    body: '隧道像一条临时通道，把别人从公网来的访问，送到你电脑里的本地服务。',
  },
  {
    id: 'public-server',
    title: '什么是公网服务器？',
    icon: 'servers',
    body: '它是一台外网能访问到的电脑，负责接住外面的请求，再转发回你的本机。',
  },
  {
    id: 'https',
    title: '什么是 HTTPS？',
    icon: 'shield-check',
    body: 'HTTPS 是带加密的网页访问方式。支付、登录、回调通常会要求使用它。',
  },
  {
    id: 'domain',
    title: '什么是域名？',
    icon: 'globe',
    body: '域名是更好记的地址，比如 api.example.com。没有域名也能用 IP 加端口访问。',
  },
  {
    id: 'certificate',
    title: '什么是证书？',
    icon: 'key',
    body: '证书用来证明这个 HTTPS 地址可信。Gate 推荐自动申请和续期。',
  },
]

export const cloudProviders: CloudProviderOption[] = [
  {
    id: 'tencent',
    name: '腾讯云',
    note: '国内访问稳定，适合微信、企微、国内支付回调。',
    tone: 'mainland',
  },
  {
    id: 'aliyun',
    name: '阿里云',
    note: '国内生态完整，适合生产环境和备案域名。',
    tone: 'mainland',
  },
  { id: 'oracle', name: 'Oracle Free', note: '有免费资源，适合预算敏感的长期体验。', tone: 'free' },
  { id: 'vultr', name: 'Vultr', note: '节点多，上手快，适合海外开发调试。', tone: 'global' },
  { id: 'hetzner', name: 'Hetzner', note: '性价比高，适合欧洲访问和自建服务。', tone: 'global' },
  {
    id: 'digitalocean',
    name: 'DigitalOcean',
    note: '文档友好，适合第一次使用云服务器。',
    tone: 'global',
  },
  {
    id: 'railway',
    name: 'Railway',
    note: '体验部署友好，适合快速验证 Web 服务。',
    tone: 'experience',
  },
  {
    id: 'zeabur',
    name: 'Zeabur',
    note: '适合轻量体验和应用托管，后续预留一键部署。',
    tone: 'experience',
  },
]

export const serverEnvironmentOptions: ServerEnvironmentOption[] = [
  {
    id: 'ubuntu',
    title: 'Ubuntu',
    icon: 'terminal',
    description: '最推荐的新手服务器环境。',
    recommendedDeploy: '推荐 Docker Compose 或 systemd 部署。',
    commandHint: '先开放 Gate 端口，再启动 server 配置。',
  },
  {
    id: 'debian',
    title: 'Debian',
    icon: 'terminal',
    description: '稳定、轻量，适合长期运行。',
    recommendedDeploy: '推荐 systemd 守护进程部署。',
    commandHint: '配置好 token 后设置开机自启。',
  },
  {
    id: 'centos',
    title: 'CentOS',
    icon: 'terminal',
    description: '常见于旧服务器和企业环境。',
    recommendedDeploy: '推荐 Docker 部署，减少系统版本差异。',
    commandHint: '注意防火墙和安全组端口放行。',
  },
  {
    id: 'windows-server',
    title: 'Windows Server',
    icon: 'monitor',
    description: '适合已有 Windows 运维环境。',
    recommendedDeploy: '推荐作为 Windows 服务运行。',
    commandHint: '确认防火墙允许 Gate server 端口。',
  },
  {
    id: 'docker',
    title: 'Docker',
    icon: 'boxes',
    description: '最容易迁移和重装。',
    recommendedDeploy: '推荐 Docker Compose。',
    commandHint: '把配置文件挂载到容器外部。',
  },
  {
    id: 'bt-panel',
    title: '宝塔',
    icon: 'sliders',
    description: '适合图形化管理 Linux 服务器。',
    recommendedDeploy: '推荐 Docker 或 Supervisor 管理。',
    commandHint: '在面板安全规则里放行公网端口。',
  },
  {
    id: '1panel',
    title: '1Panel',
    icon: 'layout-grid',
    description: '适合用面板管理容器。',
    recommendedDeploy: '推荐应用商店预留，当前使用 Docker Compose。',
    commandHint: '未来会预留一键部署入口。',
  },
  {
    id: 'casaos',
    title: 'CasaOS',
    icon: 'home',
    description: '适合家用服务器和轻量自托管。',
    recommendedDeploy: '推荐 Docker 部署。',
    commandHint: '确认路由器和云服务器链路可达。',
  },
  {
    id: 'unraid',
    title: 'Unraid',
    icon: 'hard-drive',
    description: '适合 NAS 和家庭实验室。',
    recommendedDeploy: '推荐 Docker 模板部署。',
    commandHint: '优先选择高位公网端口。',
  },
  {
    id: 'synology',
    title: 'Synology',
    icon: 'database',
    description: '预留 Synology 套件体验。',
    recommendedDeploy: '预留一键部署，当前建议 Docker。',
    commandHint: '当前只展示推荐路径。',
    reserved: true,
  },
]

export const scenarioRecommendations: ScenarioRecommendation[] = [
  {
    id: 'payment-callback',
    title: '支付回调',
    description: '支付宝、微信支付、Stripe 等回调到本地服务。',
    icon: 'credit-card',
    protocol: 'https',
    defaultName: 'payment-callback',
    localHost: '127.0.0.1',
    localPort: 8080,
    remotePort: 443,
    domainHint: 'pay.example.com',
    certificate: 'Auto',
    tags: ['Payment', 'Webhook', 'HTTPS'],
    reason: '支付平台通常要求公网 HTTPS，所以推荐域名、443 端口和自动证书。',
    domainStronglyRecommended: true,
  },
  {
    id: 'webhook',
    title: 'Webhook',
    description: 'GitHub、飞书、企业微信等事件回调。',
    icon: 'link',
    protocol: 'https',
    defaultName: 'webhook-dev',
    localHost: '127.0.0.1',
    localPort: 3000,
    remotePort: 443,
    domainHint: 'hook.example.com',
    certificate: 'Auto',
    tags: ['Webhook', 'HTTPS'],
    reason: 'Webhook 需要稳定的公网地址，HTTPS 更容易被第三方平台接受。',
  },
  {
    id: 'springboot',
    title: 'SpringBoot',
    description: '本地 Java API 或管理后台。',
    icon: 'code',
    protocol: 'https',
    defaultName: 'springboot-api',
    localHost: '127.0.0.1',
    localPort: 8080,
    remotePort: 443,
    domainHint: 'api.example.com',
    certificate: 'Auto',
    tags: ['Java', 'API', 'HTTPS'],
    reason: 'SpringBoot API 常被外部系统调用，HTTPS 和域名更接近真实环境。',
  },
  {
    id: 'vue',
    title: 'Vue 开发',
    description: 'Vite、Vue Dev Server 本地预览。',
    icon: 'code',
    protocol: 'http',
    defaultName: 'vue-dev',
    localHost: '127.0.0.1',
    localPort: 5173,
    remotePort: 18080,
    domainHint: 'vue.example.com',
    certificate: 'Not required',
    tags: ['Frontend', 'Dev'],
    reason: '前端预览优先快速访问，HTTP 加高位端口就够用，后续可切 HTTPS。',
  },
  {
    id: 'node',
    title: 'Node',
    description: 'Express、Next API、NestJS 等本地服务。',
    icon: 'file-code',
    protocol: 'https',
    defaultName: 'node-service',
    localHost: '127.0.0.1',
    localPort: 3000,
    remotePort: 443,
    domainHint: 'node.example.com',
    certificate: 'Auto',
    tags: ['Node', 'API'],
    reason: 'Node 服务常用于 API 联调，HTTPS 能减少跨平台回调限制。',
  },
  {
    id: 'python',
    title: 'Python',
    description: 'Flask、FastAPI、Django 本地服务。',
    icon: 'file-code',
    protocol: 'https',
    defaultName: 'python-api',
    localHost: '127.0.0.1',
    localPort: 8000,
    remotePort: 443,
    domainHint: 'py.example.com',
    certificate: 'Auto',
    tags: ['Python', 'API'],
    reason: 'Python API 常用于回调和模型服务，HTTPS 更适合外部系统接入。',
  },
  {
    id: 'mcp-server',
    title: 'MCP Server',
    description: '给客户端或远程 Agent 访问本地 MCP 服务。',
    icon: 'plug-zap',
    protocol: 'https',
    defaultName: 'mcp-server',
    localHost: '127.0.0.1',
    localPort: 3333,
    remotePort: 443,
    domainHint: 'mcp.example.com',
    certificate: 'Auto',
    tags: ['MCP', 'AI'],
    reason: 'MCP 服务通常给工具或 Agent 调用，HTTPS 地址更容易配置和信任。',
  },
  {
    id: 'ai-agent',
    title: 'AI Agent',
    description: '把本地 Agent API 暴露给可信调用方。',
    icon: 'sparkles',
    protocol: 'https',
    defaultName: 'agent-api',
    localHost: '127.0.0.1',
    localPort: 7860,
    remotePort: 443,
    domainHint: 'agent.example.com',
    certificate: 'Auto',
    tags: ['AI', 'Agent'],
    reason: 'Agent API 往往包含敏感能力，HTTPS 和明确域名更适合授权访问。',
  },
  {
    id: 'ssh',
    title: 'SSH',
    description: '临时远程登录本机或实验环境。',
    icon: 'terminal',
    protocol: 'tcp',
    defaultName: 'ssh-dev',
    localHost: '127.0.0.1',
    localPort: 22,
    remotePort: 2222,
    domainHint: '',
    certificate: 'Not required',
    tags: ['SSH', 'TCP'],
    reason: 'SSH 不是网页协议，推荐 TCP 隧道，并使用高位公网端口降低误扫风险。',
  },
  {
    id: 'mysql',
    title: 'MySQL',
    description: '临时访问本地 MySQL。',
    icon: 'database',
    protocol: 'tcp',
    defaultName: 'mysql-dev',
    localHost: '127.0.0.1',
    localPort: 3306,
    remotePort: 3306,
    domainHint: '',
    certificate: 'Not required',
    tags: ['Database', 'TCP'],
    reason: '数据库使用原生 TCP 连接，优先保持端口一致，便于客户端配置。',
  },
  {
    id: 'postgresql',
    title: 'PostgreSQL',
    description: '临时访问本地 PostgreSQL。',
    icon: 'database',
    protocol: 'tcp',
    defaultName: 'postgres-dev',
    localHost: '127.0.0.1',
    localPort: 5432,
    remotePort: 5432,
    domainHint: '',
    certificate: 'Not required',
    tags: ['Database', 'TCP'],
    reason: 'PostgreSQL 使用 TCP，保持默认端口能减少连接字符串改动。',
  },
  {
    id: 'redis',
    title: 'Redis',
    description: '临时访问本地 Redis。',
    icon: 'database',
    protocol: 'tcp',
    defaultName: 'redis-cache',
    localHost: '127.0.0.1',
    localPort: 6379,
    remotePort: 6379,
    domainHint: '',
    certificate: 'Not required',
    tags: ['Redis', 'TCP'],
    reason: 'Redis 是 TCP 服务，推荐只给可信网络使用，并保留默认端口。',
  },
  {
    id: 'nas',
    title: 'NAS',
    description: '访问 NAS 管理页面或文件服务。',
    icon: 'hard-drive',
    protocol: 'http',
    defaultName: 'nas-access',
    localHost: '127.0.0.1',
    localPort: 5000,
    remotePort: 15000,
    domainHint: 'nas.example.com',
    certificate: 'Not required',
    tags: ['NAS'],
    reason: 'NAS 常见管理页先用 HTTP/IP + 端口即可，正式外放再升级 HTTPS。',
  },
  {
    id: 'minecraft',
    title: 'Minecraft',
    description: 'Minecraft Java Server。',
    icon: 'box',
    protocol: 'tcp',
    defaultName: 'minecraft-server',
    localHost: '127.0.0.1',
    localPort: 25565,
    remotePort: 25565,
    domainHint: '',
    certificate: 'Not required',
    tags: ['Game', 'TCP'],
    reason: 'Minecraft Java 使用 TCP 25565，保持默认端口最容易连接。',
  },
  {
    id: 'docker',
    title: 'Docker',
    description: '容器里的 Web 服务或 Portainer。',
    icon: 'boxes',
    protocol: 'http',
    defaultName: 'docker-service',
    localHost: '127.0.0.1',
    localPort: 8080,
    remotePort: 18083,
    domainHint: 'docker.example.com',
    certificate: 'Not required',
    tags: ['Docker'],
    reason: '容器服务端口差异较大，HTTP 加可编辑端口更适合第一次验证。',
  },
  {
    id: 'custom',
    title: '自定义',
    description: '不确定场景时先建立一个通用入口。',
    icon: 'sliders',
    protocol: 'tcp',
    defaultName: 'custom-tunnel',
    localHost: '127.0.0.1',
    localPort: 8080,
    remotePort: 18080,
    domainHint: '',
    certificate: 'Not required',
    tags: ['Custom'],
    reason: '未知服务先用 TCP 打通连通性，确认协议后再调整为 HTTP 或 HTTPS。',
  },
]

export function findScenario(id: string): ScenarioRecommendation {
  return (
    scenarioRecommendations.find((scenario) => scenario.id === id) ?? scenarioRecommendations[0]
  )
}

export function findServerEnvironment(id: string): ServerEnvironmentOption | undefined {
  return serverEnvironmentOptions.find((environment) => environment.id === id)
}

export function createDefaultAnswers(): SmartWizardAnswers {
  return {
    serverOwnership: '',
    serverEnvironment: '',
    serverAddress: '',
    domainMode: '',
    domainName: '',
    scenarioId: 'springboot',
    customName: '',
    customLocalPort: null,
  }
}

export function buildSmartRecommendation(answers: SmartWizardAnswers): SmartRecommendation {
  const scenario = findScenario(answers.scenarioId)
  const environment = findServerEnvironment(answers.serverEnvironment)
  const domain = normalizeDomain(answers.domainName)
  const hasDomain = answers.domainMode === 'has-domain' && Boolean(domain)
  const localPort = answers.customLocalPort ?? scenario.localPort
  const remotePort =
    hasDomain && scenario.protocol !== 'tcp' ? scenario.remotePort : scenario.remotePort
  const server =
    normalizeServerAddress(answers.serverAddress) ||
    (answers.serverOwnership === 'has-server' ? '请填写服务器 IP 或域名' : '待绑定公网服务器')
  const protocol = scenario.protocol
  const certificate =
    protocol === 'https' ? (hasDomain ? '自动申请' : '绑定域名后自动申请') : '不需要'
  const tunnelName = answers.customName.trim() || scenario.defaultName
  const domainLabel = hasDomain ? domain : '暂不使用，先通过 IP + 端口访问'
  const publicHost = hasDomain ? domain : server
  const accessPreview =
    protocol === 'tcp'
      ? `${publicHost}:${remotePort}`
      : `${protocol}://${publicHost}${hasDomain && [80, 443].includes(remotePort) ? '' : `:${remotePort}`}`
  const deployMethod =
    environment?.recommendedDeploy ?? '选择服务器环境后，Gate 会给出推荐部署方式。'
  const domainReason = hasDomain
    ? `已使用 ${domain}，适合配置 HTTPS 和证书。`
    : '没有域名也可以使用 IP + 端口；需要支付回调时建议补充域名。'
  const serverReason =
    answers.serverOwnership === 'has-server'
      ? `${environment?.title ?? '当前服务器'} 将作为公网入口。`
      : '你还没有服务器，先生成配置草稿，拿到服务器后可重新打开引导补齐。'

  return {
    tunnelName,
    server,
    protocol,
    local: `${scenario.localHost}:${localPort}`,
    remote: String(remotePort),
    domain: domainLabel,
    certificate,
    accessPreview,
    deployMethod,
    reasonList: [scenario.reason, domainReason, serverReason],
    tags: scenario.tags,
    form: {
      name: tunnelName,
      protocol,
      localHost: scenario.localHost,
      localPort,
      remotePort,
      projectId: 'p1',
      serverName: server,
      autoStart: false,
      remark: scenario.reason,
      tags: [...scenario.tags],
    },
  }
}

export function normalizeServerAddress(value: string): string {
  return value
    .trim()
    .replace(/^https?:\/\//, '')
    .replace(/\/$/, '')
}

export function normalizeDomain(value: string): string {
  return value
    .trim()
    .replace(/^https?:\/\//, '')
    .replace(/\/.*$/, '')
}
