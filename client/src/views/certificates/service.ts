import { TauriIpcClient } from '@/ipc'
import type { CertificateDetailResponse, CertificateListResponse } from './types'

const ipc = new TauriIpcClient()

export const certificateService = {
  list() {
    return ipc.invoke<CertificateListResponse>('certificate_list')
  },
  detail(domain: string) {
    return ipc.invoke<CertificateDetailResponse>('certificate_detail', { domain })
  },
  exportPem(domain: string) {
    return ipc.invoke<string>('certificate_export_pem', { domain })
  },
}
