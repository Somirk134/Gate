import { GateAppError, TauriIpcClient } from '@/ipc'
import type { DashboardData, HealthReport } from '@/monitoring/types'
import type { PortDiscovery } from './discovery.service'
import type { ServerFormData } from '@/views/servers/types'
import { isTauri } from '@tauri-apps/api/core'

const ipc = new TauriIpcClient()

export interface RuntimeServerRecord {
  id: string
  name: string
  kind: string
  host: string
  port: number
  token: string
  region: string
  remark: string
  tags: string[]
  heartbeatInterval: number
  reconnectInterval: number
  autoConnect: boolean
  status: string
  lastError?: string | null
  lastRttMs?: number | null
  sessionId?: string | null
  lastCheckedAt?: number | null
  lastConnectedAt?: number | null
  discovery?: ServerDiscovery | null
  createdAt: number
  updatedAt: number
}

export interface ServerDiscovery {
  serverName?: string
  hostname?: string
  os?: string
  cpu?: string
  memory?: {
    totalBytes?: number
    usedBytes?: number
  }
  architecture?: string
  publicIp?: string
  privateIp?: string
  docker?: { detected?: boolean }
  firewall?: { detected?: boolean }
  gateVersion?: string
  runtimeVersion?: string
  uptime?: number
  latency?: number
  region?: string
  diskUsage?: {
    totalBytes?: number
    usedBytes?: number
    availableBytes?: number
  }
  networkUsage?: {
    receivedBytes?: number
    transmittedBytes?: number
  }
  portDiscovery?: PortDiscovery
  discoveredAt?: number
}

export interface RuntimeServerList {
  items: RuntimeServerRecord[]
  activeServerId?: string | null
  connected: boolean
}

export interface ServerTestResult {
  ok: boolean
  rttMs?: number
  sessionId?: string
  checkedAt: number
  error?: string
}

function ensureRuntime() {
  if (!isTauri()) {
    throw new GateAppError({
      code: 'SERVER_RUNTIME_UNAVAILABLE',
      messageKey: 'errors.serverRuntimeUnavailable',
      timestamp: Date.now(),
    })
  }
}

function toRequest(form: ServerFormData) {
  return {
    name: form.name,
    kind: form.kind,
    host: form.host,
    port: form.port ?? 0,
    token: form.token,
    region: form.region,
    remark: form.remark,
    tags: form.tags,
    heartbeatInterval: form.heartbeatInterval,
    reconnectInterval: form.reconnectInterval,
    autoConnect: form.autoConnect,
  }
}

export const serverService = {
  async list() {
    ensureRuntime()
    return ipc.invoke<RuntimeServerList>('server_list')
  },

  async create(form: ServerFormData) {
    ensureRuntime()
    return ipc.invoke<string>('server_create', { request: toRequest(form) })
  },

  async update(id: string, patch: Partial<ServerFormData>) {
    ensureRuntime()
    return ipc.invoke<void>('server_update', {
      serverId: id,
      patch: {
        name: patch.name,
        kind: patch.kind,
        host: patch.host,
        port: patch.port ?? undefined,
        token: patch.token,
        region: patch.region,
        remark: patch.remark,
        tags: patch.tags,
        heartbeatInterval: patch.heartbeatInterval,
        reconnectInterval: patch.reconnectInterval,
        autoConnect: patch.autoConnect,
      },
    })
  },

  async remove(id: string) {
    ensureRuntime()
    return ipc.invoke<void>('server_delete', { serverId: id })
  },

  async connect(id: string) {
    ensureRuntime()
    return ipc.invoke<string>('server_connect', { serverId: id })
  },

  async disconnect(id: string) {
    ensureRuntime()
    return ipc.invoke<void>('server_disconnect', { serverId: id })
  },

  async test(id: string) {
    ensureRuntime()
    return ipc.invoke<ServerTestResult>('server_test', { serverId: id })
  },

  async status() {
    ensureRuntime()
    const dashboard = await ipc.invoke<DashboardData>('runtime_get_dashboard')
    return dashboard.serverStatus
  },

  async health() {
    ensureRuntime()
    return ipc.invoke<HealthReport>('runtime_get_health')
  },
}
