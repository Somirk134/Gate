export type CertificateStatus =
  'pending' | 'active' | 'expiringSoon' | 'expired' | 'revoked' | 'deleted' | 'failed' | 'unknown'

export type AutoRenewalStatus = 'scheduled' | 'due' | 'notScheduled' | 'expired' | 'failed'

export interface CertificateSummary {
  domain: string
  issuer: string
  createTime: string
  expireTime: string
  renewTime?: string | null
  daysRemaining: number
  status: CertificateStatus
  autoRenewalStatus: AutoRenewalStatus
  fingerprintSha256: string
  algorithm: string
  san: string[]
  serialNumber?: string | null
  lastError?: string | null
  hasCertificatePem: boolean
  certificatePath: string
  keyPath: string
}

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
