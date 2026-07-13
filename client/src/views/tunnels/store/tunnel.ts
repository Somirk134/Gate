import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { i18n } from '@/i18n'
import { GateAppError } from '@/ipc'
import { tunnelService } from '@/services/tunnel.service'
import { useServerStore } from '@views/servers'
import type { DashboardTunnel } from '@/monitoring/types'
import type {
  Tunnel,
  TunnelFormData,
  TunnelLoadStatus,
  TunnelProtocol,
  TunnelStatus,
} from '../types'
import {
  TUNNEL_STATUS_CONFIG,
  alignPublicPortWithProtocol,
  buildTunnelPublicUrl,
  isRunningStatus,
} from '../utils'

function t(key: string, params?: Record<string, unknown>): string {
  return (i18n.global as unknown as { t: (key: string, params?: Record<string, unknown>) => string }).t(
    key,
    params,
  )
}

export const defaultTunnelForm: TunnelFormData = {
  name: '',
  protocol: 'tcp',
  localHost: '127.0.0.1',
  localPort: null,
  remotePort: null,
  host: '',
  path: '/',
  projectId: '',
  serverId: '',
  serverName: '',
  autoStart: false,
  remark: '',
  tags: [],
}

const RUNNING_GRACE_MS = 30_000

export const useTunnelStore = defineStore('tunnel-module', () => {
  const tunnels = ref<Tunnel[]>([])
  const status = ref<TunnelLoadStatus>('idle')
  const error = ref<string>('')
  const lastUpdated = ref<number>(0)
  const pendingActionIds = ref<Set<string>>(new Set())
  const runningGraceUntil = ref<Record<string, number>>({})

  const isLoading = computed(() => status.value === 'loading')
  const isError = computed(() => status.value === 'error')
  const isReady = computed(() => status.value === 'success')
  const hasTunnels = computed(() => tunnels.value.length > 0)

  const runningTunnels = computed(() => tunnels.value.filter((t) => isRunningStatus(t.status)))
  const stoppedTunnels = computed(() =>
    tunnels.value.filter((t) => t.status === 'stopped' || t.status === 'offline'),
  )
  const httpTunnels = computed(() => tunnels.value.filter((t) => t.protocol === 'http'))
  const tcpTunnels = computed(() => tunnels.value.filter((t) => t.protocol === 'tcp'))

  const totalConnections = computed(() =>
    tunnels.value.reduce((sum, t) => sum + t.statistics.connections, 0),
  )
  const totalTraffic = computed(() =>
    tunnels.value.reduce((sum, t) => sum + t.traffic.total, 0),
  )
  const totalUploadSpeed = computed(() =>
    tunnels.value.reduce((sum, t) => sum + t.traffic.uploadSpeed, 0),
  )
  const totalDownloadSpeed = computed(() =>
    tunnels.value.reduce((sum, t) => sum + t.traffic.downloadSpeed, 0),
  )

  function getById(id: string): Tunnel | undefined {
    return tunnels.value.find((t) => t.id === id)
  }

  function isActionPending(id: string): boolean {
    return pendingActionIds.value.has(id)
  }

  function setTunnelStatusLocal(id: string, nextStatus: TunnelStatus) {
    const index = tunnels.value.findIndex((tunnel) => tunnel.id === id)
    if (index === -1) return
    const current = tunnels.value[index]
    if (current.status === nextStatus) return
    tunnels.value[index] = { ...current, status: nextStatus }
  }

  function markRunningGrace(id: string) {
    runningGraceUntil.value = { ...runningGraceUntil.value, [id]: Date.now() + RUNNING_GRACE_MS }
  }

  function clearRunningGrace(id: string) {
    if (!(id in runningGraceUntil.value)) return
    const next = { ...runningGraceUntil.value }
    delete next[id]
    runningGraceUntil.value = next
  }

  function reconcileRunningGrace(id: string) {
    const deadline = runningGraceUntil.value[id]
    if (!deadline) return

    const tunnel = getById(id)
    if (!tunnel || Date.now() >= deadline) {
      clearRunningGrace(id)
      return
    }

    if (isRunningStatus(tunnel.status)) {
      clearRunningGrace(id)
      return
    }

    setTunnelStatusLocal(id, 'running')
  }

  function reconcileAllRunningGrace() {
    for (const id of Object.keys(runningGraceUntil.value)) {
      reconcileRunningGrace(id)
    }
  }

  function withPendingAction<T>(id: string, action: () => Promise<T>): Promise<T> {
    pendingActionIds.value = new Set([...pendingActionIds.value, id])
    return action().finally(() => {
      const next = new Set(pendingActionIds.value)
      next.delete(id)
      pendingActionIds.value = next
    })
  }

  async function load(options: { silent?: boolean } = {}): Promise<void> {
    const silent = options.silent === true && status.value === 'success'
    if (!silent) {
      status.value = 'loading'
      error.value = ''
    }
    try {
      const rows = await tunnelService.list()
      tunnels.value = rows.map(mapRuntimeTunnel)
      reconcileAllRunningGrace()
      status.value = 'success'
      error.value = ''
      lastUpdated.value = Date.now()
    } catch (e) {
      error.value = e instanceof Error ? e.message : t('tunnel.errors.loadFailed')
      if (!silent) {
        status.value = 'error'
      }
    }
  }

  async function refresh(): Promise<void> {
    return load({ silent: true })
  }

  async function createTunnel(form: TunnelFormData): Promise<Tunnel> {
    const normalizedForm = normalizeTunnelFormForSave(form)
    const serverStore = useServerStore()
    if (serverStore.status === 'idle') {
      await serverStore.load()
    }
    const selectedServer =
      serverStore.servers.find((server) => server.id === normalizedForm.serverId) ??
      serverStore.servers.find((server) => server.name === normalizedForm.serverName)
    if (!selectedServer || selectedServer.status !== 'connected') {
      throw new GateAppError({
        code: 'TUNNEL_SERVER_REQUIRED',
        messageKey: 'tunnel.errors.needConnectedServer',
        timestamp: Date.now(),
      })
    }

    const id = await tunnelService.create({
      localPort: normalizedForm.localPort ?? 0,
      remotePort: normalizedForm.remotePort ?? 0,
      protocol: normalizedForm.protocol,
      serverId: selectedServer.id,
      localHost: normalizedForm.localHost || '127.0.0.1',
      host: optionalText(normalizedForm.host),
      path: optionalText(normalizedForm.path),
    })

    await tunnelService.edit(id, {
      name: normalizedForm.name.trim(),
      protocol: normalizedForm.protocol,
      serverId: selectedServer.id,
      localHost: normalizedForm.localHost || '127.0.0.1',
      localPort: normalizedForm.localPort ?? 0,
      remotePort:
        normalizedForm.remotePort && normalizedForm.remotePort > 0
          ? normalizedForm.remotePort
          : undefined,
      host: optionalText(normalizedForm.host),
      path: optionalText(normalizedForm.path),
    })

    await refresh()
    const created = getById(id)
    if (!created) {
      throw new GateAppError({
        code: 'TUNNEL_RELOAD_FAILED',
        messageKey: 'tunnel.errors.savedReloadFailed',
        details: { tunnelId: id },
        timestamp: Date.now(),
      })
    }

    if (normalizedForm.autoStart) {
      await startTunnel(created.id)
      return getById(created.id) ?? created
    }

    return created
  }

  async function updateTunnel(id: string, patch: Partial<TunnelFormData>): Promise<void> {
    const previous = getById(id)
    const shouldRestart = previous ? isRunningStatus(previous.status) : false
    const normalizedPatch = normalizeTunnelPatchForSave(previous, patch)
    const nextProtocol = normalizedPatch.protocol ?? previous?.protocol
    const keepsHttpFields = nextProtocol === 'http' || nextProtocol === 'https'
    await tunnelService.edit(id, {
      name: normalizedPatch.name,
      protocol: normalizedPatch.protocol,
      serverId: normalizedPatch.serverId,
      localHost: normalizedPatch.localHost,
      localPort: normalizedPatch.localPort ?? undefined,
      remotePort: normalizedPatch.remotePort ?? undefined,
      host:
        patch.host === undefined
          ? undefined
          : keepsHttpFields
            ? patch.host.trim()
            : '',
      path:
        patch.path === undefined
          ? undefined
          : keepsHttpFields
            ? patch.path.trim()
            : '',
    })

    if (shouldRestart) {
      // 修改运行中隧道后主动重启转发，让端口和本地目标立即生效。
      try {
        await tunnelService.restart(id)
      } finally {
        await refresh()
      }
      return
    }

    await refresh()
  }

  async function removeTunnel(id: string): Promise<void> {
    await tunnelService.delete(id)
    await refresh()
  }

  async function startTunnel(id: string): Promise<void> {
    return withPendingAction(id, async () => {
      setTunnelStatusLocal(id, 'starting')
      try {
        await tunnelService.start(id)
        markRunningGrace(id)
        setTunnelStatusLocal(id, 'running')
        await refresh()
      } catch (error) {
        clearRunningGrace(id)
        await refresh()
        throw error
      }
    })
  }

  async function stopTunnel(id: string): Promise<void> {
    return withPendingAction(id, async () => {
      clearRunningGrace(id)
      setTunnelStatusLocal(id, 'stopping')
      try {
        await tunnelService.stop(id)
        setTunnelStatusLocal(id, 'stopped')
        await refresh()
      } catch (error) {
        await refresh()
        throw error
      }
    })
  }

  async function cancelTunnel(id: string): Promise<void> {
    clearRunningGrace(id)
    setTunnelStatusLocal(id, 'stopping')
    try {
      await tunnelService.stop(id)
      setTunnelStatusLocal(id, 'stopped')
      await refresh()
    } catch (error) {
      await refresh()
      throw error
    }
  }

  async function restartTunnel(id: string): Promise<void> {
    return withPendingAction(id, async () => {
      setTunnelStatusLocal(id, 'restarting')
      try {
        await tunnelService.restart(id)
        markRunningGrace(id)
        setTunnelStatusLocal(id, 'running')
        await refresh()
      } catch (error) {
        clearRunningGrace(id)
        await refresh()
        throw error
      }
    })
  }

  return {
    tunnels,
    status,
    error,
    lastUpdated,
    isLoading,
    isError,
    isReady,
    hasTunnels,
    runningTunnels,
    stoppedTunnels,
    httpTunnels,
    tcpTunnels,
    totalConnections,
    totalTraffic,
    totalUploadSpeed,
    totalDownloadSpeed,
    getById,
    isActionPending,
    load,
    refresh,
    createTunnel,
    updateTunnel,
    removeTunnel,
    startTunnel,
    stopTunnel,
    cancelTunnel,
    restartTunnel,
  }
})

function mapRuntimeTunnel(row: DashboardTunnel): Tunnel {
  const protocol = normalizeProtocol(row.protocol)
  const status = normalizeStatus(row.status)

  return {
    id: row.id,
    name: row.name,
    protocol,
    localHost: row.localHost ?? '127.0.0.1',
    localPort: row.localPort ?? 0,
    remotePort: row.remotePort ?? 0,
    host: row.host ?? null,
    path: row.path ?? null,
    publicAddr: publicAddress(row),
    remark: '',
    status,
    autoStart: false,
    tags: [],
    serverId: row.serverId ?? '',
    serverName: row.serverName ?? '',
    projectName: '',
    projectId: '',
    traffic: {
      uploadSpeed: row.uploadSpeedBps,
      downloadSpeed: row.downloadSpeedBps,
      total: row.trafficBytes ?? 0,
      history: [],
    },
    statistics: {
      uptime: row.uptimeSeconds,
      connections: row.connections,
      totalConnections: row.connections,
      requests: row.requestCount ?? 0,
      avgLatency: row.averageResponseTimeMs ?? 0,
      peakSpeed: Math.max(row.uploadSpeedBps, row.downloadSpeedBps),
    },
    connections: [],
    logs: (row.recentLogs ?? []).map((log) => ({
      id: `${row.id}-${log.source}-${log.timestamp}-${log.message}`,
      level: normalizeLogLevel(log.level),
      message: log.message,
      timestamp: log.timestamp,
      source: log.source,
    })),
    lastStartedAt: row.uptimeSeconds > 0 ? `${row.uptimeSeconds}s` : '',
  }
}

function normalizeProtocol(protocol: DashboardTunnel['protocol']): TunnelProtocol {
  if (protocol === 'http' || protocol === 'tcp' || protocol === 'https') {
    return protocol
  }
  return 'tcp'
}

function normalizeStatus(status: DashboardTunnel['status']): TunnelStatus {
  if (status === 'running') return 'running'
  if (status === 'starting') return 'starting'
  if (status === 'stopping') return 'stopping'
  if (status === 'restarting') return 'restarting'
  if (status === 'recovering') return 'connecting'
  if (status === 'error' || status === 'warning') return 'error'
  return 'stopped'
}

function normalizeLogLevel(level: string): Tunnel['logs'][number]['level'] {
  if (
    level === 'debug' ||
    level === 'info' ||
    level === 'warn' ||
    level === 'error' ||
    level === 'success'
  ) {
    return level
  }
  return 'info'
}

function publicAddress(row: DashboardTunnel): string {
  const runtimeAddress = row.publicAddress?.trim()
  if (runtimeAddress) return runtimeAddress

  return buildTunnelPublicUrl({
    protocol: row.protocol,
    host: row.host,
    path: row.path,
    remotePort: row.remotePort,
    serverHost: row.publicHost,
  }) || t('tunnel.notAssigned')
}

function optionalText(value: string | undefined): string | undefined {
  const trimmed = value?.trim()
  return trimmed ? trimmed : undefined
}

function normalizeTunnelFormForSave(form: TunnelFormData): TunnelFormData {
  const remotePort =
    form.protocol === 'http' || form.protocol === 'https'
      ? alignPublicPortWithProtocol(form.protocol, form.remotePort)
      : form.remotePort

  return {
    ...form,
    remotePort,
  }
}

function normalizeTunnelPatchForSave(
  previous: Tunnel | undefined,
  patch: Partial<TunnelFormData>,
): Partial<TunnelFormData> {
  const protocol = patch.protocol ?? previous?.protocol
  if (protocol !== 'http' && protocol !== 'https') {
    return patch
  }

  const remotePort = alignPublicPortWithProtocol(
    protocol,
    patch.remotePort ?? previous?.remotePort ?? null,
  )

  return {
    ...patch,
    remotePort,
  }
}

export { TUNNEL_STATUS_CONFIG }
