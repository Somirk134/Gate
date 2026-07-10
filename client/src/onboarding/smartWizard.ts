import type { TunnelFormData, TunnelProtocol } from '@/views/tunnels/types'

export type ServerOwnership = 'has-server' | 'no-server' | 'unknown-server'
export type DomainMode = 'has-domain' | 'no-domain' | 'skip-domain'
export type DeployMode = 'linux-vps' | 'docker'
export type TranslateFn = (key: string, params?: Record<string, unknown>) => string

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
  localeKey: string
  icon: string
}

export interface CloudProviderOption {
  id: string
  localeKey: string
  tone: 'mainland' | 'global' | 'free' | 'experience'
}

export interface ServerEnvironmentOption {
  id: ServerEnvironmentId
  localeKey: string
  icon: string
  reserved?: boolean
}

export interface ScenarioRecommendation {
  id: string
  localeKey: string
  icon: string
  protocol: Extract<TunnelProtocol, 'tcp' | 'http' | 'https'>
  defaultName: string
  localHost: string
  localPort: number
  remotePort: number
  domainHint: string
  certificate: 'auto' | 'notRequired'
  tags: string[]
  domainStronglyRecommended?: boolean
}

export interface SmartWizardAnswers {
  serverOwnership: ServerOwnership | ''
  serverEnvironment: ServerEnvironmentId | ''
  serverAddress: string
  serverPort: number
  serverToken: string
  deployMode: DeployMode
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
  { id: 'tunnel', localeKey: 'tunnel', icon: 'router' },
  { id: 'public-server', localeKey: 'publicServer', icon: 'servers' },
  { id: 'https', localeKey: 'https', icon: 'shield-check' },
  { id: 'domain', localeKey: 'domain', icon: 'globe' },
  { id: 'certificate', localeKey: 'certificate', icon: 'key' },
]

export const cloudProviders: CloudProviderOption[] = [
  { id: 'tencent', localeKey: 'tencent', tone: 'mainland' },
  { id: 'aliyun', localeKey: 'aliyun', tone: 'mainland' },
  { id: 'oracle', localeKey: 'oracle', tone: 'free' },
  { id: 'vultr', localeKey: 'vultr', tone: 'global' },
  { id: 'hetzner', localeKey: 'hetzner', tone: 'global' },
  { id: 'digitalocean', localeKey: 'digitalocean', tone: 'global' },
  { id: 'railway', localeKey: 'railway', tone: 'experience' },
  { id: 'zeabur', localeKey: 'zeabur', tone: 'experience' },
]

export const serverEnvironmentOptions: ServerEnvironmentOption[] = [
  { id: 'ubuntu', localeKey: 'ubuntu', icon: 'terminal' },
  { id: 'debian', localeKey: 'debian', icon: 'terminal' },
  { id: 'centos', localeKey: 'centos', icon: 'terminal' },
  { id: 'windows-server', localeKey: 'windowsServer', icon: 'monitor' },
  { id: 'docker', localeKey: 'docker', icon: 'boxes' },
  { id: 'bt-panel', localeKey: 'btPanel', icon: 'sliders' },
  { id: '1panel', localeKey: 'onePanel', icon: 'layout-grid' },
  { id: 'casaos', localeKey: 'casaos', icon: 'home' },
  { id: 'unraid', localeKey: 'unraid', icon: 'hard-drive' },
  { id: 'synology', localeKey: 'synology', icon: 'database', reserved: true },
]

export const scenarioRecommendations: ScenarioRecommendation[] = [
  {
    id: 'payment-callback',
    localeKey: 'paymentCallback',
    icon: 'credit-card',
    protocol: 'https',
    defaultName: 'payment-callback',
    localHost: '127.0.0.1',
    localPort: 8080,
    remotePort: 443,
    domainHint: 'pay.example.com',
    certificate: 'auto',
    tags: ['Payment', 'Webhook', 'HTTPS'],
    domainStronglyRecommended: true,
  },
  {
    id: 'webhook',
    localeKey: 'webhook',
    icon: 'link',
    protocol: 'https',
    defaultName: 'webhook-dev',
    localHost: '127.0.0.1',
    localPort: 3000,
    remotePort: 443,
    domainHint: 'hook.example.com',
    certificate: 'auto',
    tags: ['Webhook', 'HTTPS'],
  },
  {
    id: 'springboot',
    localeKey: 'springboot',
    icon: 'code',
    protocol: 'https',
    defaultName: 'springboot-api',
    localHost: '127.0.0.1',
    localPort: 8080,
    remotePort: 443,
    domainHint: 'api.example.com',
    certificate: 'auto',
    tags: ['Java', 'API', 'HTTPS'],
  },
  {
    id: 'vue',
    localeKey: 'vue',
    icon: 'code',
    protocol: 'http',
    defaultName: 'vue-dev',
    localHost: '127.0.0.1',
    localPort: 5173,
    remotePort: 18080,
    domainHint: 'vue.example.com',
    certificate: 'notRequired',
    tags: ['Frontend', 'Dev'],
  },
  {
    id: 'node',
    localeKey: 'node',
    icon: 'file-code',
    protocol: 'https',
    defaultName: 'node-service',
    localHost: '127.0.0.1',
    localPort: 3000,
    remotePort: 443,
    domainHint: 'node.example.com',
    certificate: 'auto',
    tags: ['Node', 'API'],
  },
  {
    id: 'python',
    localeKey: 'python',
    icon: 'file-code',
    protocol: 'https',
    defaultName: 'python-api',
    localHost: '127.0.0.1',
    localPort: 8000,
    remotePort: 443,
    domainHint: 'py.example.com',
    certificate: 'auto',
    tags: ['Python', 'API'],
  },
  {
    id: 'mcp-server',
    localeKey: 'mcpServer',
    icon: 'plug-zap',
    protocol: 'https',
    defaultName: 'mcp-server',
    localHost: '127.0.0.1',
    localPort: 3333,
    remotePort: 443,
    domainHint: 'mcp.example.com',
    certificate: 'auto',
    tags: ['MCP', 'AI'],
  },
  {
    id: 'ai-agent',
    localeKey: 'aiAgent',
    icon: 'sparkles',
    protocol: 'https',
    defaultName: 'agent-api',
    localHost: '127.0.0.1',
    localPort: 7860,
    remotePort: 443,
    domainHint: 'agent.example.com',
    certificate: 'auto',
    tags: ['AI', 'Agent'],
  },
  {
    id: 'ssh',
    localeKey: 'ssh',
    icon: 'terminal',
    protocol: 'tcp',
    defaultName: 'ssh-dev',
    localHost: '127.0.0.1',
    localPort: 22,
    remotePort: 2222,
    domainHint: '',
    certificate: 'notRequired',
    tags: ['SSH', 'TCP'],
  },
  {
    id: 'mysql',
    localeKey: 'mysql',
    icon: 'database',
    protocol: 'tcp',
    defaultName: 'mysql-dev',
    localHost: '127.0.0.1',
    localPort: 3306,
    remotePort: 3306,
    domainHint: '',
    certificate: 'notRequired',
    tags: ['Database', 'TCP'],
  },
  {
    id: 'postgresql',
    localeKey: 'postgresql',
    icon: 'database',
    protocol: 'tcp',
    defaultName: 'postgres-dev',
    localHost: '127.0.0.1',
    localPort: 5432,
    remotePort: 5432,
    domainHint: '',
    certificate: 'notRequired',
    tags: ['Database', 'TCP'],
  },
  {
    id: 'redis',
    localeKey: 'redis',
    icon: 'database',
    protocol: 'tcp',
    defaultName: 'redis-cache',
    localHost: '127.0.0.1',
    localPort: 6379,
    remotePort: 6379,
    domainHint: '',
    certificate: 'notRequired',
    tags: ['Redis', 'TCP'],
  },
  {
    id: 'nas',
    localeKey: 'nas',
    icon: 'hard-drive',
    protocol: 'http',
    defaultName: 'nas-access',
    localHost: '127.0.0.1',
    localPort: 5000,
    remotePort: 15000,
    domainHint: 'nas.example.com',
    certificate: 'notRequired',
    tags: ['NAS'],
  },
  {
    id: 'minecraft',
    localeKey: 'minecraft',
    icon: 'box',
    protocol: 'tcp',
    defaultName: 'minecraft-server',
    localHost: '127.0.0.1',
    localPort: 25565,
    remotePort: 25565,
    domainHint: '',
    certificate: 'notRequired',
    tags: ['Game', 'TCP'],
  },
  {
    id: 'docker',
    localeKey: 'docker',
    icon: 'boxes',
    protocol: 'http',
    defaultName: 'docker-service',
    localHost: '127.0.0.1',
    localPort: 8080,
    remotePort: 18083,
    domainHint: 'docker.example.com',
    certificate: 'notRequired',
    tags: ['Docker'],
  },
  {
    id: 'custom',
    localeKey: 'custom',
    icon: 'sliders',
    protocol: 'tcp',
    defaultName: 'custom-tunnel',
    localHost: '127.0.0.1',
    localPort: 8080,
    remotePort: 18080,
    domainHint: '',
    certificate: 'notRequired',
    tags: ['Custom'],
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
    serverPort: 7000,
    // 发布版本不提供共享默认口令，必须由用户填写服务端实际配置。
    serverToken: '',
    deployMode: 'linux-vps',
    domainMode: '',
    domainName: '',
    scenarioId: 'springboot',
    customName: '',
    customLocalPort: null,
  }
}

export function buildSmartRecommendation(
  answers: SmartWizardAnswers,
  t: TranslateFn,
): SmartRecommendation {
  const scenario = findScenario(answers.scenarioId)
  const environment = findServerEnvironment(answers.serverEnvironment)
  const domain = normalizeDomain(answers.domainName)
  const hasDomain = answers.domainMode === 'has-domain' && Boolean(domain)
  const localPort = answers.customLocalPort ?? scenario.localPort
  const remotePort =
    hasDomain && scenario.protocol !== 'tcp' ? scenario.remotePort : scenario.remotePort
  const server =
    normalizeServerAddress(answers.serverAddress) ||
    (answers.serverOwnership === 'has-server'
      ? t('welcome.recommendation.serverMissing')
      : t('welcome.recommendation.serverPending'))
  const protocol = scenario.protocol
  const certificate =
    protocol === 'https'
      ? hasDomain
        ? t('welcome.recommendation.certificateAuto')
        : t('welcome.recommendation.certificateAfterDomain')
      : t('welcome.recommendation.certificateNotRequired')
  const tunnelName = answers.customName.trim() || scenario.defaultName
  const domainLabel = hasDomain ? domain : t('welcome.recommendation.noDomainAccess')
  const publicHost = hasDomain ? domain : server
  const accessPreview =
    protocol === 'tcp'
      ? `${publicHost}:${remotePort}`
      : `${protocol}://${publicHost}${hasDomain && [80, 443].includes(remotePort) ? '' : `:${remotePort}`}`
  const deployMethod = environment
    ? t(`welcome.environments.${environment.localeKey}.recommendedDeploy`)
    : t('welcome.recommendation.deployAfterEnvironment')
  const domainReason = hasDomain
    ? t('welcome.recommendation.domainReasonWithDomain', { domain })
    : t('welcome.recommendation.domainReasonWithoutDomain')
  const serverReason =
    answers.serverOwnership === 'has-server'
      ? t('welcome.recommendation.serverReason', {
          environment: environment
            ? t(`welcome.environments.${environment.localeKey}.title`)
            : t('welcome.recommendation.currentServer'),
        })
      : t('welcome.recommendation.noServerReason')
  const scenarioReason = t(`welcome.scenarios.${scenario.localeKey}.reason`)

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
    reasonList: [scenarioReason, domainReason, serverReason],
    tags: scenario.tags,
    form: {
      name: tunnelName,
      protocol,
      localHost: scenario.localHost,
      localPort,
      remotePort,
      projectId: 'p1',
      serverId: '',
      serverName: server,
      autoStart: false,
      remark: scenarioReason,
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
