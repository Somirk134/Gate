import { TauriIpcClient } from '@/ipc'
import type {
  AcmePrepareRequest,
  AcmePrepareResponse,
  AcmeVerifyResponse,
  AutoRenewalStatusResponse,
  CertificateDetailResponse,
  CertificateDomainAssociations,
  CertificateListResponse,
  CertificateOperationResult,
  CertificateStats,
  ImportRequest,
  ImportResponse,
  ImportValidation,
} from './types'

const ipc = new TauriIpcClient()

export const certificateService = {
  /* ── 查询 ── */
  list() {
    return ipc.invoke<CertificateListResponse>('certificate_list')
  },
  detail(domain: string) {
    return ipc.invoke<CertificateDetailResponse>('certificate_detail', { domain })
  },
  exportPem(domain: string) {
    return ipc.invoke<string>('certificate_export_pem', { domain })
  },
  stats() {
    return ipc.invoke<CertificateStats>('certificate_stats')
  },

  /* ── 导入 ── */
  validateImport(certificatePem: string, privateKeyPem: string) {
    return ipc.invoke<ImportValidation>('certificate_validate_import', {
      certificatePem,
      privateKeyPem,
    })
  },
  importCertificate(request: ImportRequest) {
    return ipc.invoke<ImportResponse>('certificate_import', { request })
  },

  /* ── 操作 ── */
  delete(domain: string) {
    return ipc.invoke<CertificateOperationResult>('certificate_delete', { domain })
  },
  renewNow(domain: string) {
    return ipc.invoke<CertificateOperationResult>('certificate_renew_now', { domain })
  },
  redeploy(domain: string) {
    return ipc.invoke<CertificateOperationResult>('certificate_redeploy', { domain })
  },
  toggleAutoRenewal(domain: string, enabled: boolean) {
    return ipc.invoke<CertificateOperationResult>('certificate_toggle_auto_renewal', {
      domain,
      enabled,
    })
  },

  /* ── 自动续期 ── */
  autoRenewalStatus() {
    return ipc.invoke<AutoRenewalStatusResponse>('certificate_auto_renewal_status')
  },

  /* ── 域名关联 ── */
  domainAssociations(domain: string) {
    return ipc.invoke<CertificateDomainAssociations>('certificate_domain_associations', { domain })
  },

  /* ── ACME 申请流程 ── */
  acmePrepare(request: AcmePrepareRequest) {
    return ipc.invoke<AcmePrepareResponse>('certificate_acme_prepare', { request })
  },
  acmeVerify() {
    return ipc.invoke<AcmeVerifyResponse>('certificate_acme_verify')
  },
}
