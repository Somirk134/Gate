export const DOMAIN_HEALTH_TONES: Record<string, string> = {
  healthy: 'success',
  warning: 'warning',
  offline: 'neutral',
  expired: 'error',
  dnsError: 'error',
  certificateError: 'error',
  tunnelOffline: 'warning',
}

export function formatDomainBytes(value: number): string {
  if (!Number.isFinite(value) || value <= 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  let size = value
  let index = 0
  while (size >= 1024 && index < units.length - 1) {
    size /= 1024
    index += 1
  }
  return `${size >= 10 || index === 0 ? size.toFixed(0) : size.toFixed(1)} ${units[index]}`
}

export function formatRelativeTime(timestamp: number | null | undefined): string {
  if (!timestamp) return '-'
  const delta = Date.now() - timestamp
  if (delta < 60_000) return '<1m'
  if (delta < 3_600_000) return `${Math.floor(delta / 60_000)}m`
  if (delta < 86_400_000) return `${Math.floor(delta / 3_600_000)}h`
  return `${Math.floor(delta / 86_400_000)}d`
}
