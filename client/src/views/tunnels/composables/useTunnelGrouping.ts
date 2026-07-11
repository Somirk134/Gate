import { computed, ref, watch, type Ref } from 'vue'
import type { ComposerTranslation } from 'vue-i18n'
import type { Tunnel, TunnelGroup, TunnelGroupMode } from '../types'

const COLLAPSED_STORAGE_KEY = 'gate:tunnel-group-collapsed'

function isRunning(status: Tunnel['status']): boolean {
  return (
    status === 'running' ||
    status === 'starting' ||
    status === 'restarting' ||
    status === 'connecting'
  )
}

function runningCount(tunnels: Tunnel[]): number {
  return tunnels.filter((tunnel) => isRunning(tunnel.status)).length
}

function groupKey(mode: TunnelGroupMode, value: string): string {
  return `${mode}:${value}`
}

function readCollapsedKeys(): Set<string> {
  try {
    const raw = localStorage.getItem(COLLAPSED_STORAGE_KEY)
    if (!raw) return new Set()
    const parsed = JSON.parse(raw)
    return Array.isArray(parsed) ? new Set(parsed.filter((item) => typeof item === 'string')) : new Set()
  } catch {
    return new Set()
  }
}

function writeCollapsedKeys(keys: Set<string>) {
  localStorage.setItem(COLLAPSED_STORAGE_KEY, JSON.stringify([...keys]))
}

export function buildTunnelGroups(
  tunnels: Tunnel[],
  mode: TunnelGroupMode,
  t: ComposerTranslation,
): TunnelGroup[] {
  if (mode === 'none' || !tunnels.length) return []

  const buckets = new Map<string, { label: string; tunnels: Tunnel[] }>()

  const ensureBucket = (key: string, label: string) => {
    const existing = buckets.get(key)
    if (existing) return existing
    const bucket = { label, tunnels: [] as Tunnel[] }
    buckets.set(key, bucket)
    return bucket
  }

  for (const tunnel of tunnels) {
    if (mode === 'project') {
      const label = tunnel.projectName?.trim() || t('tunnel.group.uncategorizedProject')
      ensureBucket(groupKey(mode, label), label).tunnels.push(tunnel)
      continue
    }

    if (mode === 'server') {
      const label = tunnel.serverName?.trim() || t('tunnel.group.uncategorizedServer')
      ensureBucket(groupKey(mode, label), label).tunnels.push(tunnel)
      continue
    }

    if (mode === 'protocol') {
      const label = tunnel.protocol.toUpperCase()
      ensureBucket(groupKey(mode, label), label).tunnels.push(tunnel)
      continue
    }

    if (mode === 'tag') {
      if (!tunnel.tags.length) {
        const label = t('tunnel.group.uncategorizedTag')
        ensureBucket(groupKey(mode, label), label).tunnels.push(tunnel)
        continue
      }
      for (const tag of tunnel.tags) {
        const label = tag
        ensureBucket(groupKey(mode, tag), label).tunnels.push(tunnel)
      }
    }
  }

  return [...buckets.entries()]
    .map(([key, bucket]) => ({
      key,
      label: bucket.label,
      tunnels: bucket.tunnels,
      runningCount: runningCount(bucket.tunnels),
    }))
    .sort((left, right) => left.label.localeCompare(right.label, undefined, { sensitivity: 'base' }))
}

export function useTunnelGrouping(
  tunnels: Ref<Tunnel[]>,
  groupMode: Ref<TunnelGroupMode>,
  t: ComposerTranslation,
) {
  const collapsedKeys = ref<Set<string>>(readCollapsedKeys())

  const groups = computed(() => buildTunnelGroups(tunnels.value, groupMode.value, t))

  watch(
    collapsedKeys,
    (keys) => {
      writeCollapsedKeys(keys)
    },
    { deep: true },
  )

  function isCollapsed(key: string): boolean {
    return collapsedKeys.value.has(key)
  }

  function toggleGroup(key: string) {
    const next = new Set(collapsedKeys.value)
    if (next.has(key)) {
      next.delete(key)
    } else {
      next.add(key)
    }
    collapsedKeys.value = next
  }

  return {
    groups,
    isCollapsed,
    toggleGroup,
  }
}
