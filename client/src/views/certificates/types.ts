/* ────────────────────────── 证书状态 ────────────────────────── */

export type CertificateStatus =
  | 'pending'
  | 'active'
  | 'expiringSoon'
  | 'expired'
  | 'revoked'
  | 'deleted'
  | 'failed'
  | 'unknown'

export type AutoRenewalStatus = 'scheduled' | 'due' | 'notScheduled' | 'expired' | 'failed'

export type DeployStatus = 'deployed' | 'pending' | 'failed'

export type CertSortType = 'updatedAt' | 'expireTime' | 'domain' | 'status'
export type CertFilterType = 'all' | 'active' | 'expiringSoon' | 'expired' | 'failed'
export type SortDirection = 'asc' | 'desc'

/* ────────────────────────── 证书摘要 ────────────────────────── */

export interface CertificateSummary {
  domain: string
  issuer: string
  createTime: string
  expireTime: string
  renewTime?: string | null
  daysRemaining: number
  status: CertificateStatus
  autoRenewalStatus: AutoRenewalStatus
  autoRenewalEnabled: boolean
  fingerprintSha256: string
  algorithm: string
  san: string[]
  serialNumber?: string | null
  lastError?: string | null
  hasCertificatePem: boolean
  certificatePath: string
  keyPath: string
  tlsVersion: string
  deployStatus: DeployStatus
}

/* ────────────────────────── API 响应 ────────────────────────── */

export interface CertificateListResponse {
  storeRoot: string
  certificates: CertificateSummary[]
  generatedAt: number
}

export interface CertificateDetailResponse {
  summary: CertificateSummary
  record: Record<string, unknown>
  certificatePem: string
  generatedAt: number
}

/* ────────────────────────── 统计与健康 ────────────────────────── */

export interface CertificateStats {
  total: number
  active: number
  expiringSoon: number
  expired: number
  failed: number
  autoRenewalOk: number
  autoRenewalFailed: number
  healthScore: number
  statusDistribution: {
    active: number
    expiringSoon: number
    expired: number
    failed: number
  }
  healthChecks: {
    autoRenewal: boolean
    acme: boolean
    dns: boolean
    http01: boolean
    tls13: boolean
    sni: boolean
  }
  generatedAt: number
}

/* ────────────────────────── 自动续期 ────────────────────────── */

export interface AutoRenewalStatusResponse {
  enabled: boolean
  acmeEmail?: string | null
  acmeStaging: boolean
  acmeDirectoryUrl?: string | null
  acmeHttp01Port: number
  checkIntervalSeconds: number
  renewBeforeDays: number
  scheduleDescription: string
  lastRenewTime?: string | null
  lastRenewSuccess: boolean
  lastError?: string | null
  nextCheckHours: number
  generatedAt: number
}

/* ────────────────────────── 域名关联 ────────────────────────── */

export interface DomainAssociation {
  host: string
  tunnelId?: string | null
  status: string
  verifyStatus: string
}

export interface CertificateDomainAssociations {
  domain: string
  domains: DomainAssociation[]
  tunnels: { tunnelId: string; domain: string }[]
  projects: string[]
  dbAvailable: boolean
  dbPath: string
  generatedAt: number
}

/* ────────────────────────── 导入验证 ────────────────────────── */

export interface ImportValidation {
  valid: boolean
  commonName: string
  issuer: string
  san: string[]
  algorithm: string
  serialNumber: string
  fingerprintSha256: string
  notBefore: string
  notAfter: string
  isExpired: boolean
  daysRemaining: number
  keyValid: boolean
  keyType: string
  tlsSupported: boolean
}

export interface ImportRequest {
  domain: string
  certificatePem: string
  privateKeyPem: string
}

export interface ImportResponse {
  domain: string
  imported: boolean
  validation: ImportValidation
  path: string
}

/* ────────────────────────── 操作结果 ────────────────────────── */

export interface CertificateOperationResult {
  domain: string
  triggered?: boolean
  deleted?: boolean
  imported?: boolean
  autoRenewalEnabled?: boolean
  message?: string
}

/* ────────────────────────── Wizard ────────────────────────── */

export interface CertificateWizardForm {
  serverId: string
  domain: string
  email: string
  challengeType: 'http01' | 'dns01'
  staging: boolean
}

export type WizardStep = 'server' | 'domain' | 'email' | 'verify' | 'request' | 'result'

export interface WizardCheckItem {
  key: string
  label: string
  status: 'pending' | 'checking' | 'success' | 'failed'
  detail?: string
}

/* ────────────────────────── ACME 流程 ────────────────────────── */

export interface AcmePrepareRequest {
  domain: string
  email: string
  challengeType: 'http01' | 'dns01'
  staging: boolean
}

export interface AcmePrepareResponse {
  domain: string
  challengeType: string
  txtHost: string
  txtValue: string
  http01Token: string
  http01Path: string
  http01Content: string
  directoryUrl: string
  staging: boolean
  generatedAt: number
}

export interface AcmeVerifyResponse {
  domain: string
  success: boolean
  certificatePem: string
  generatedAt: number
}
