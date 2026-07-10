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
  AcmeHistoryResponse,
  AcmeRecordDetailResponse,
  AcmeRetryResponse,
  AcmeDeleteRecordResponse,
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

  /** 后台启动 ACME 验证（非阻塞，通过事件返回结果） */
  startAcmeVerify(recordId?: string) {
    return ipc.invoke<{ domain: string; started: boolean; message: string; recordId?: string }>('certificate_acme_start_verify', { recordId })
  },

  /* ── 申请历史 ── */
  /** 获取所有 ACME 申请记录 */
  history() {
    return ipc.invoke<AcmeHistoryResponse>('certificate_acme_history')
  },

  /** 获取单条申请记录详情（含证书信息） */
  recordDetail(recordId: string) {
    return ipc.invoke<AcmeRecordDetailResponse>('certificate_acme_record_detail', { recordId })
  },

  /** 重试失败的或正在验证的申请 */
  retryApplication(recordId: string) {
    return ipc.invoke<AcmeRetryResponse>('certificate_acme_retry', { recordId })
  },

  /** 删除一条申请记录（不删除证书文件） */
  deleteRecord(recordId: string) {
    return ipc.invoke<AcmeDeleteRecordResponse>('certificate_acme_delete_record', { recordId })
  },
}
