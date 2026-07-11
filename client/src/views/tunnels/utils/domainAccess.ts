import type { CertificateSummary } from '@views/certificates/types'
import type { TunnelProtocol } from '../types'

export const SUBDOMAIN_PREFIX_PRESETS = ['dev', 'api', 'app', 'test', 'staging'] as const

export interface DnsRecordGuide {
  type: 'A' | 'CNAME'
  name: string
  value: string
  host: string
}

export function isApexDomain(domain: string): boolean {
  const normalized = domain.trim().toLowerCase()
  if (!normalized || normalized.startsWith('*.')) return false
  return normalized.split('.').filter(Boolean).length === 2
}

export function suggestSubdomainPrefix(baseDomain: string): string {
  return isApexDomain(baseDomain) ? 'dev' : baseDomain.split('.')[0] || 'dev'
}

export function buildSubdomainHost(prefix: string, baseDomain: string): string {
  const normalizedPrefix = prefix.trim().toLowerCase().replace(/\.$/, '')
  const normalizedBase = baseDomain.trim().toLowerCase()
  if (!normalizedBase) return ''
  if (!normalizedPrefix || normalizedPrefix === '@') return normalizedBase
  if (normalizedBase.startsWith(`${normalizedPrefix}.`)) return normalizedBase
  return `${normalizedPrefix}.${normalizedBase}`
}

export function splitSubdomainHost(
  host: string,
  baseDomains: string[],
): { prefix: string; baseDomain: string } | null {
  const normalized = host.trim().toLowerCase()
  if (!normalized) return null
  const matchedBase = [...baseDomains]
    .sort((left, right) => right.length - left.length)
    .find((base) => normalized === base || normalized.endsWith(`.${base}`))
  if (!matchedBase) return null
  if (normalized === matchedBase) {
    return { prefix: '', baseDomain: matchedBase }
  }
  return {
    prefix: normalized.slice(0, -(matchedBase.length + 1)),
    baseDomain: matchedBase,
  }
}

export function certificateCoversHost(host: string, certs: CertificateSummary[] = []): boolean {
  const normalizedHost = host.trim().toLowerCase()
  if (!normalizedHost) return false
  for (const cert of certs) {
    if (cert.status !== 'active' && cert.status !== 'expiringSoon') continue
    for (const domain of [cert.domain, ...cert.san]) {
      const normalizedDomain = domain.trim().toLowerCase()
      if (!normalizedDomain) continue
      if (normalizedDomain === normalizedHost) return true
      if (normalizedDomain.startsWith('*.')) {
        const suffix = normalizedDomain.slice(2)
        if (normalizedHost === suffix || normalizedHost.endsWith(`.${suffix}`)) {
          return true
        }
      }
    }
  }
  return false
}

export function listCertificateBaseDomains(certs: CertificateSummary[] = []): string[] {
  const seen = new Set<string>()
  const domains: string[] = []
  for (const cert of certs) {
    if (cert.status !== 'active' && cert.status !== 'expiringSoon') continue
    for (const domain of [cert.domain, ...cert.san]) {
      const normalized = domain.trim().toLowerCase()
      if (!normalized || normalized.startsWith('*.') || seen.has(normalized)) continue
      seen.add(normalized)
      domains.push(normalized)
    }
  }
  return domains.sort((left, right) => left.localeCompare(right))
}

export function suggestedSubdomainHosts(certs: CertificateSummary[] = []): string[] {
  return listCertificateBaseDomains(certs).flatMap((base) =>
    SUBDOMAIN_PREFIX_PRESETS.map((prefix) => buildSubdomainHost(prefix, base)),
  )
}

export function isHttpLikeProtocol(protocol: string): boolean {
  return protocol === 'http' || protocol === 'https'
}

/** 默认公网端口：避免与服务器上 Nginx 等占用的 80/443 冲突。 */
export const DEFAULT_HTTP_PUBLIC_PORT = 8880
export const DEFAULT_HTTPS_PUBLIC_PORT = 8443

export function standardPublicPort(protocol: TunnelProtocol | string): number {
  return protocol === 'https' ? DEFAULT_HTTPS_PUBLIC_PORT : DEFAULT_HTTP_PUBLIC_PORT
}

export function isLegacyStandardPublicPort(port: number): boolean {
  return port === 80 || port === 443
}

/** 仅修正空端口或 80/443 特权端口，保留用户自定义高位端口（如 HTTP 使用 8443）。 */
export function alignPublicPortWithProtocol(
  protocol: TunnelProtocol | string,
  remotePort: number | null | undefined,
): number {
  if (!isHttpLikeProtocol(protocol)) {
    return remotePort ?? 0
  }

  const port = remotePort ?? 0
  if (port < 1 || port > 65535) {
    return standardPublicPort(protocol)
  }
  if (isLegacyStandardPublicPort(port)) {
    return standardPublicPort(protocol)
  }
  return port
}

/** 仅在 HTTP ↔ HTTPS 协议切换时，同步默认端口对。 */
export function alignPublicPortOnProtocolSwitch(
  protocol: TunnelProtocol | string,
  previousProtocol: TunnelProtocol | string,
  remotePort: number | null | undefined,
): number {
  const port = remotePort ?? 0
  if (previousProtocol === 'http' && protocol === 'https' && port === DEFAULT_HTTP_PUBLIC_PORT) {
    return DEFAULT_HTTPS_PUBLIC_PORT
  }
  if (previousProtocol === 'https' && protocol === 'http' && port === DEFAULT_HTTPS_PUBLIC_PORT) {
    return DEFAULT_HTTP_PUBLIC_PORT
  }
  return alignPublicPortWithProtocol(protocol, port)
}

export function usesSubdomainAccess(protocol: string, host?: string | null): boolean {
  return isHttpLikeProtocol(protocol) && Boolean(host?.trim())
}

export function normalizeTunnelPath(path?: string | null): string {
  const value = path?.trim()
  if (!value || value === '/') return '/'
  return value.startsWith('/') ? value : `/${value}`
}

export function formatTunnelPathForUrl(path?: string | null): string {
  const normalized = normalizeTunnelPath(path)
  return normalized === '/' ? '' : normalized
}

export function shouldOmitPublicPort(protocol: string, remotePort: number, host?: string | null): boolean {
  if (!isHttpLikeProtocol(protocol) || !host?.trim()) return false
  return isLegacyStandardPublicPort(remotePort)
}

export function buildTunnelPublicUrl(options: {
  protocol: string
  host?: string | null
  path?: string | null
  remotePort?: number | null
  serverHost?: string | null
}): string {
  const protocol = options.protocol
  const pathSuffix = formatTunnelPathForUrl(options.path)
  const host = options.host?.trim().toLowerCase()
  const serverHost = options.serverHost?.trim()
  const remotePort = options.remotePort ?? 0

  if (isHttpLikeProtocol(protocol)) {
    if (host) {
      const effectivePort = remotePort > 0 ? remotePort : standardPublicPort(protocol)
      const portSuffix = shouldOmitPublicPort(protocol, effectivePort, host) ? '' : `:${effectivePort}`
      return `${protocol}://${host}${portSuffix}${pathSuffix}`
    }
    if (serverHost && remotePort > 0) {
      const portSuffix = shouldOmitPublicPort(protocol, remotePort, host) ? '' : `:${remotePort}`
      return `${protocol}://${serverHost}${portSuffix}${pathSuffix}`
    }
  }

  if (serverHost && remotePort > 0) {
    return `${serverHost}:${remotePort}`
  }
  if (remotePort > 0) {
    return `:${remotePort}`
  }
  return ''
}

export function buildDnsRecordGuide(host: string, serverIp: string): DnsRecordGuide | null {
  const normalizedHost = host.trim().toLowerCase()
  const normalizedIp = serverIp.trim()
  if (!normalizedHost || !normalizedIp) return null

  const labels = normalizedHost.split('.').filter(Boolean)
  const name = labels.length > 2 ? labels[0] : '@'

  return {
    type: 'A',
    name,
    value: normalizedIp,
    host: normalizedHost,
  }
}

export function applySubdomainTunnelDefaults(protocol: TunnelProtocol | string, host: string): {
  host: string
  path: string
  remotePort: number
} {
  return {
    host: host.trim().toLowerCase(),
    path: '/',
    remotePort: standardPublicPort(protocol),
  }
}
