import { GateAppError, TauriIpcClient } from '@/ipc'
import { isTauri } from '@tauri-apps/api/core'
import type {
  DomainBatchPayload,
  DomainCreatePayload,
  DomainDetailResponse,
  DomainDnsSnapshot,
  DomainListQuery,
  DomainListResponse,
  DomainStats,
  DomainTopologyResponse,
} from '../types'

const ipc = new TauriIpcClient()

function ensureRuntime() {
  if (!isTauri()) {
    throw new GateAppError({
      code: 'RUNTIME_UNAVAILABLE',
      messageKey: 'errors.runtimeUnavailable',
      timestamp: Date.now(),
    })
  }
}

export const domainService = {
  list(query: DomainListQuery = {}) {
    ensureRuntime()
    return ipc.invoke<DomainListResponse>('domain_list_command', { query })
  },

  stats() {
    ensureRuntime()
    return ipc.invoke<DomainStats>('domain_stats_command')
  },

  detail(host: string) {
    ensureRuntime()
    return ipc.invoke<DomainDetailResponse>('domain_detail_command', { host })
  },

  checkDns(host: string) {
    ensureRuntime()
    return ipc.invoke<DomainDnsSnapshot>('domain_check_dns_command', { host })
  },

  create(request: DomainCreatePayload) {
    ensureRuntime()
    return ipc.invoke<DomainDetailResponse>('domain_create_command', { request })
  },

  delete(host: string) {
    ensureRuntime()
    return ipc.invoke<{ host: string; deleted: boolean }>('domain_delete_command', { host })
  },

  bindTunnel(host: string, tunnelId: string) {
    ensureRuntime()
    return ipc.invoke<{ host: string; bound: boolean }>('domain_bind_tunnel_command', {
      request: { host, tunnelId },
    })
  },

  unbindTunnel(host: string) {
    ensureRuntime()
    return ipc.invoke('domain_unbind_tunnel_command', { host })
  },

  batch(request: DomainBatchPayload) {
    ensureRuntime()
    return ipc.invoke('domain_batch_command', { request })
  },

  topology() {
    ensureRuntime()
    return ipc.invoke<DomainTopologyResponse>('domain_topology_command')
  },
}
