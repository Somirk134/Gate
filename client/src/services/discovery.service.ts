import { GateAppError, TauriIpcClient } from '@/ipc'
import { isTauri } from '@tauri-apps/api/core'

const ipc = new TauriIpcClient()

export interface LocalServiceRecord {
  port: number
  process: string
  protocol: string
  executable: string
  pid?: number | null
  technology: string
  recommendedProtocol: 'tcp' | 'http' | 'https'
  host: string
  bindAddress?: string
  label: string
  reachable?: boolean
  manual?: boolean
}

export interface PortRecord {
  port: number
  pid?: number | null
  processName?: string
  protocol: string
  status: string
  executable?: string
  recommended?: boolean
}

export interface PortDiscovery {
  occupiedPorts: PortRecord[]
  availablePorts: PortRecord[]
  systemReservedPorts: PortRecord[]
  gateReservedPorts: PortRecord[]
  updatedAt: number
}

export interface TunnelDiagnosis {
  ok: boolean
  summary: string
  findings: Array<{
    id: string
    status: 'ok' | 'warning' | 'error'
    reason: string
    solution: string
  }>
  checkedAt: number
}

function ensureRuntime() {
  if (!isTauri()) {
    throw new GateAppError({
      code: 'DISCOVERY_RUNTIME_UNAVAILABLE',
      messageKey: 'errors.runtimeUnavailable',
      timestamp: Date.now(),
    })
  }
}

export const discoveryService = {
  async localServices() {
    ensureRuntime()
    const payload = await ipc.invoke<{ items: LocalServiceRecord[]; updatedAt: number }>(
      'discovery_local_services',
    )
    return payload.items
  },

  async probeLocalService(host: string, port: number) {
    ensureRuntime()
    return ipc.invoke<LocalServiceRecord>('discovery_probe_local_service', { host, port })
  },

  async remotePorts(serverId?: string | null) {
    ensureRuntime()
    return ipc.invoke<PortDiscovery>('discovery_remote_ports', { serverId })
  },

  async checkRemotePort(port: number, serverId?: string | null) {
    ensureRuntime()
    return ipc.invoke<{ port: number; available: boolean; status: string; reason: string }>(
      'discovery_check_remote_port',
      { serverId, port },
    )
  },

  async diagnoseTunnel(data: {
    localHost: string
    localPort: number
    remotePort: number
    serverId?: string | null
  }) {
    ensureRuntime()
    return ipc.invoke<TunnelDiagnosis>('discovery_diagnose_tunnel', {
      localHost: data.localHost,
      localPort: data.localPort,
      remotePort: data.remotePort,
      serverId: data.serverId,
    })
  },
}
