import { i18n } from '@/i18n'
import { GateAppError } from '@/ipc'

type Diagnosis = {
  key: string
  params?: Record<string, string | number>
}

function t(key: string, params?: Record<string, string | number>): string {
  const global = i18n.global as unknown as {
    t: (k: string, named?: Record<string, string | number>) => string
    te: (k: string) => boolean
  }
  if (!global.te(key)) {
    return key
  }
  try {
    return global.t(key, params)
  } catch {
    return key
  }
}

function extractSource(err: unknown): string {
  if (err instanceof GateAppError) {
    const source = err.details?.source
    if (typeof source === 'string' && source.trim()) {
      return source.trim()
    }
    if (
      err.messageKey.startsWith('tunnel.errors.') &&
      err.messageKey !== 'errors.tunnel.operationFailed' &&
      err.messageKey !== 'tunnel.errors.operationFailedDetail'
    ) {
      return ''
    }
  }
  if (typeof err === 'string') return err.trim()
  if (err instanceof Error && err.message.trim()) return err.message.trim()
  return ''
}

function parseHostPort(text: string): { host?: string; port?: number } {
  const match = text.match(/(\d{1,3}(?:\.\d{1,3}){3}|localhost|127\.0\.0\.1|[\w.-]+):(\d{2,5})/i)
  if (!match) return {}
  return { host: match[1], port: Number(match[2]) }
}

function parseServerName(text: string): string | undefined {
  const match = text.match(/`([^`]+)`/)
  return match?.[1]
}

function diagnoseTunnelSource(source: string): Diagnosis | null {
  if (!source) return null

  const lower = source.toLowerCase()

  if (source === 'tunnel not found') {
    return { key: 'tunnel.errors.notFound' }
  }
  if (source === 'SERVER_SESSION_MISSING') {
    return { key: 'tunnel.errors.sessionExpired' }
  }
  if (source === 'NO_AVAILABLE_SERVER_CONFIG') {
    return { key: 'tunnel.errors.noServer' }
  }
  if (source === 'SERVER_NOT_FOUND_FOR_TUNNEL' || source === 'SERVER_REQUIRED_FOR_TUNNEL') {
    return { key: 'tunnel.errors.configServerMissing' }
  }
  if (source.startsWith('REMOTE_PORT_OCCUPIED:')) {
    const port = source.split(':').pop() ?? ''
    return { key: 'tunnel.errors.remotePortOccupied', params: { port } }
  }
  if (source.includes('REMOTE_PORT_REQUIRED')) {
    return { key: 'tunnel.errors.remotePortRequired' }
  }
  if (source.includes('LOCAL_PORT_REQUIRED')) {
    return { key: 'tunnel.errors.localPortRequired' }
  }
  if (
    source.includes('SERVER_DISCONNECTED_TUNNEL_START_BLOCKED') ||
    lower.includes('no active connection') ||
    lower.includes('server is disconnected') ||
    lower.includes('runtime backend is not connected')
  ) {
    const name = parseServerName(source)
    const endpoint = parseHostPort(source)
    if (name && endpoint.host && endpoint.port) {
      return {
        key: 'tunnel.errors.serverOfflineNamed',
        params: { name, host: endpoint.host, port: endpoint.port },
      }
    }
    return { key: 'tunnel.errors.serverOffline' }
  }
  if (
    lower.includes('local service') &&
    (lower.includes('unreachable') || lower.includes('timed out') || lower.includes('refused'))
  ) {
    const endpoint = parseHostPort(source)
    if (endpoint.host && endpoint.port) {
      return {
        key: 'tunnel.errors.localServiceUnreachable',
        params: { host: endpoint.host, port: endpoint.port },
      }
    }
    return { key: 'tunnel.errors.localServiceUnreachableGeneric' }
  }
  if (lower.includes('failed to resolve local service address') || lower.includes('address is invalid')) {
    const endpoint = parseHostPort(source)
    if (endpoint.host && endpoint.port) {
      return {
        key: 'tunnel.errors.localServiceInvalid',
        params: { host: endpoint.host, port: endpoint.port },
      }
    }
    return { key: 'tunnel.errors.localServiceInvalidGeneric' }
  }
  if (
    lower.includes('already in use') ||
    lower.includes('10048') ||
    lower.includes('only one usage')
  ) {
    const endpoint = parseHostPort(source)
    return {
      key: 'tunnel.errors.localPortOccupied',
      params: { port: endpoint.port ?? '' },
    }
  }
  if (lower.includes('token') || lower.includes('auth')) {
    return { key: 'tunnel.errors.authFailed' }
  }
  if (lower.includes('certificate') || lower.includes('acme')) {
    return { key: 'tunnel.errors.certificateFailed' }
  }
  if (lower.includes('server unavailable') || lower.includes('control connection disconnected')) {
    return { key: 'tunnel.errors.serverControlFailed' }
  }
  if (lower.includes('local runtime failed to start')) {
    const endpoint = parseHostPort(source)
    if (endpoint.host && endpoint.port) {
      return {
        key: 'tunnel.errors.localRuntimeFailed',
        params: { host: endpoint.host, port: endpoint.port },
      }
    }
    return { key: 'tunnel.errors.localRuntimeFailedGeneric' }
  }

  return null
}

export function formatTunnelOperationError(err: unknown, fallbackKey = 'tunnel.errors.unknown'): string {
  if (err instanceof GateAppError) {
    const { messageKey, details } = err
    if (
      messageKey.startsWith('tunnel.errors.') &&
      messageKey !== 'tunnel.errors.operationFailedDetail'
    ) {
      const params: Record<string, string | number> = {}
      for (const [key, value] of Object.entries(details ?? {})) {
        if (key === 'source') continue
        if (typeof value === 'string' || typeof value === 'number') {
          params[key] = value
        }
      }
      const translated = t(messageKey, params)
      if (translated && translated !== messageKey) {
        return translated
      }
    }
  }

  const source = extractSource(err)
  const diagnosis = diagnoseTunnelSource(source)
  if (diagnosis) {
    return t(diagnosis.key, diagnosis.params)
  }

  if (source) {
    return t('tunnel.errors.operationFailedDetail', { source })
  }

  if (err instanceof GateAppError && err.message && err.message !== err.messageKey) {
    return err.message
  }
  if (err instanceof Error && err.message.trim()) {
    return err.message
  }

  return t(fallbackKey)
}
