export type DomainHealthStatus =
  | 'healthy'
  | 'warning'
  | 'offline'
  | 'expired'
  | 'dnsError'
  | 'certificateError'
  | 'tunnelOffline'

export type DomainDnsStatus = 'matched' | 'mismatched' | 'noRecord' | 'notChecked' | 'error' | 'unknown'

export interface DomainCertificateSummary {
  domain: string
  issuer: string
  status: string
  expireTime: number
  daysRemaining: number
  san: string[]
  tlsVersion?: string | null
  autoRenewalEnabled: boolean
  renewTime?: number | null
}

export interface ManagedDomainRecord {
  id: string
  host: string
  aliases: string[]
  protocol: string
  path: string
  tunnelId: string
  tunnelName: string
  projectId: string
  projectName: string
  serverId: string
  serverName: string
  serverHost: string
  https: boolean
  certificate: DomainCertificateSummary | null
  dnsStatus: DomainDnsStatus
  healthStatus: DomainHealthStatus
  lastAccessAt: number | null
  requestCount24h: number
  traffic24h: number
  requestTrend: number[]
  trafficTrend: number[]
  createdAt: number
  updatedAt: number
  status: string
  url: string
}

export interface DomainListResponse {
  items: ManagedDomainRecord[]
  total: number
  page: number
  pageSize: number
  generatedAt: number
  dbPath: string
}

export interface DomainStats {
  total: number
  online: number
  https: number
  http: number
  expiringSoon: number
  abnormal: number
  unboundTunnel: number
  dnsFailed: number
  requests24h: number
  traffic24h: number
  requestTrend: number[]
  trafficTrend: number[]
  generatedAt: number
}

export interface DomainDnsRecord {
  type: string
  values: string[]
  ttl: number
  resolvedToServer: boolean
  status: string
}

export interface DomainDnsSnapshot {
  host: string
  status: DomainDnsStatus
  resolvedToServer: boolean
  serverAddresses: string[]
  records: DomainDnsRecord[]
  checkedAt: number
}

export interface DomainRuntimeDetail {
  httpRequests: number
  trafficBytes: number
  latencyMs: number
  errorRate: number
  currentConnections: number
  peakConnections: number
  requestTrend: unknown[]
  latencyTrend: unknown[]
}

export interface DomainDetailResponse {
  summary: ManagedDomainRecord
  managed: Record<string, unknown> | null
  dns: DomainDnsSnapshot
  runtime: DomainRuntimeDetail
  logs: {
    access: Array<Record<string, unknown>>
    error: Array<Record<string, unknown>>
  }
  generatedAt: number
}

export interface DomainTopologyNode {
  id: string
  type: 'project' | 'tunnel' | 'domain' | 'certificate' | 'https'
  label: string
  route: string
}

export interface DomainTopologyEdge {
  from: string
  to: string
}

export interface DomainTopologyResponse {
  nodes: DomainTopologyNode[]
  edges: DomainTopologyEdge[]
  generatedAt: number
}

export interface DomainListQuery {
  keyword?: string
  health?: string
  protocol?: string
  sortBy?: string
  sortDir?: 'asc' | 'desc'
  page?: number
  pageSize?: number
}

export interface DomainCreatePayload {
  host: string
  tunnelId?: string
  protocol?: string
  path?: string
  projectId?: string
}

export interface DomainBatchPayload {
  hosts: string[]
  action: string
  tunnelId?: string
  certificateDomain?: string
}
