export type ProjectStatus = 'running' | 'partial' | 'stopped' | 'starting' | 'error'

export type ProjectColor =
  | 'blue'
  | 'green'
  | 'purple'
  | 'orange'
  | 'red'
  | 'cyan'
  | 'pink'
  | 'indigo'
  | 'teal'
  | 'amber'
  | 'slate'

export type ProjectTemplate =
  | 'blank'
  | 'springBoot'
  | 'vue'
  | 'node'
  | 'python'
  | 'docker'
  | 'mcpServer'
  | 'aiAgent'
  | 'nas'
  | 'ssh'
  | 'git'
  | 'webhook'
  | 'custom'

export interface ProjectEnvironmentVariable {
  key: string
  value: string
  secret: boolean
}

export interface ProjectEnvironment {
  id: string
  name: string
  variables: ProjectEnvironmentVariable[]
}

export interface ProjectNote {
  id: string
  title: string
  content: string
  createdAt: number
  updatedAt: number
}

export interface ProjectRecord {
  id: string
  name: string
  description: string
  icon: string
  color: ProjectColor
  template: ProjectTemplate
  tunnelIds: string[]
  domains: string[]
  certificateIds: string[]
  tags: string[]
  environments: ProjectEnvironment[]
  notes: ProjectNote[]
  favorite: boolean
  pinned: boolean
  autoStart: boolean
  startupPolicy?: string | null
  lastActivityAt: number
  createdAt: number
  updatedAt: number
}

export interface TunnelRecommendation {
  id: string
  name: string
  protocol: 'tcp' | 'http' | 'https' | string
  localHost: string
  localPort: number
  remotePort: number
  description: string
  tags: string[]
}

export interface ProjectTemplateProfile {
  key: ProjectTemplate
  label: string
  icon: string
  color: ProjectColor
  description: string
  tags: string[]
  recommendations: TunnelRecommendation[]
}

export interface ProjectTag {
  id: string
  name: string
}

export interface ProjectLogEntry {
  id: string
  level: 'debug' | 'info' | 'warn' | 'error' | 'success' | string
  source: string
  message: string
  timestamp: number
  tunnelId?: string | null
}

export interface ProjectStatistics {
  todayTraffic: number
  totalTraffic: number
  uptime: number
  connections: number
  tunnelCount: number
  runningTunnelCount: number
  httpsCount: number
  tcpCount: number
  requestCount: number
  tlsSessionCount: number
  errorCount: number
  averageLatencyMs: number
  bandwidthBps: number
  cpuUsage: number
  memoryUsage: number
}

export interface Project {
  id: string
  name: string
  description: string
  icon: string
  color: ProjectColor
  template: ProjectTemplate
  tags: string[]
  serverName: string
  autoStart: boolean
  startupPolicy?: string | null
  remark?: string
  status: ProjectStatus
  pinned: boolean
  favorite: boolean
  lastUsedAt: number
  lastActivityAt: number
  tunnelIds: string[]
  domains: string[]
  certificateIds: string[]
  environments: ProjectEnvironment[]
  notes: ProjectNote[]
  tunnelCount: number
  runningTunnelCount: number
  domainCount: number
  certificateCount: number
  statistics: ProjectStatistics
  recentLogs: ProjectLogEntry[]
  recentErrors: ProjectLogEntry[]
  certificateStatus: 'healthy' | 'warning' | 'missing' | 'unknown'
  lastStartedAt: string
  createdAt: number
  updatedAt: number
}

export type ProjectFilterType = 'all' | 'running' | 'stopped' | 'favorite' | 'recent'

export type ProjectSortType = 'name' | 'createdAt' | 'updatedAt' | 'status' | 'tunnelCount'

export type SortDirection = 'asc' | 'desc'

export interface ProjectFormData {
  name: string
  icon: string
  color: ProjectColor
  template: ProjectTemplate
  description: string
  serverName: string
  autoStart: boolean
  tags: string[]
  remark: string
  environments: ProjectEnvironment[]
  startupPolicy?: string | null
}

export interface ColorPreset {
  key: ProjectColor
  label: string
  value: string
}

export interface IconPreset {
  key: string
  label: string
}

export interface TagPreset {
  name: string
  color: string
}

export interface ProjectDeleteImpact {
  tunnelCount: number
  domainCount: number
  certificateCount: number
  hasReferences: boolean
}

export type ProjectDeleteMode = 'projectOnly' | 'cascadeResources'

export interface ProjectDeleteResponse {
  projectId: string
  impact: ProjectDeleteImpact
  deletedTunnelIds: string[]
  failedTunnelIds: string[]
}

export type ProjectLoadStatus = 'idle' | 'loading' | 'success' | 'error'

export interface ProjectTunnelSummary {
  id: string
  name: string
  protocol: 'tcp' | 'http' | 'https'
  localAddr: string
  remoteAddr: string
  publicAddr: string
  status: 'online' | 'offline' | 'starting' | 'error'
  downSpeed: string
  upSpeed: string
  connections: number
}
