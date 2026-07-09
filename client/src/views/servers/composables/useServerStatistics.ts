/* ==================================================================
   useServerStatistics — 服务器统计聚合组合式函数
   ------------------------------------------------------------------
   从服务器集合派生汇总统计，供工具栏 / 概览 / Dashboard 复用。
   ================================================================== */

import { computed, type Ref } from 'vue'
import type { Server } from '../types'
import { isOnlineStatus } from '../utils'

export function useServerStatistics(servers: Ref<Server[]>) {
  const total = computed(() => servers.value.length)
  const online = computed(() => servers.value.filter((s) => isOnlineStatus(s.status)))
  const offline = computed(() =>
    servers.value.filter((s) => s.status === 'offline' || s.status === 'disconnected'),
  )
  const errorCount = computed(() => servers.value.filter((s) => s.status === 'error').length)
  const unhealthyCount = computed(
    () => servers.value.filter((s) => s.health.overall !== 'healthy').length,
  )

  const onlineCount = computed(() => online.value.length)
  const offlineCount = computed(() => offline.value.length)

  const totalTunnels = computed(() =>
    servers.value.reduce((sum, s) => sum + s.statistics.tunnelCount, 0),
  )
  const totalProjects = computed(() =>
    servers.value.reduce((sum, s) => sum + s.statistics.projectCount, 0),
  )
  const totalConnections = computed(() =>
    servers.value.reduce((sum, s) => sum + s.monitor.connections.active, 0),
  )
  const totalTraffic = computed(() =>
    servers.value.reduce((sum, s) => sum + s.traffic.totalUpload + s.traffic.totalDownload, 0),
  )
  const totalUploadSpeed = computed(() =>
    servers.value.reduce((sum, s) => sum + s.traffic.uploadSpeed, 0),
  )
  const totalDownloadSpeed = computed(() =>
    servers.value.reduce((sum, s) => sum + s.traffic.downloadSpeed, 0),
  )
  const avgPing = computed(() => {
    const r = online.value
    if (r.length === 0) return 0
    return Math.round(r.reduce((sum, s) => sum + s.ping, 0) / r.length)
  })
  const avgHealth = computed(() => {
    if (servers.value.length === 0) return 0
    return Math.round(
      servers.value.reduce((sum, s) => sum + s.health.score, 0) / servers.value.length,
    )
  })

  /** 类型分布 */
  const kindDistribution = computed(() => {
    const counts: Record<string, number> = {}
    for (const s of servers.value) {
      counts[s.kind] = (counts[s.kind] ?? 0) + 1
    }
    return Object.entries(counts).map(([key, value]) => ({
      key,
      label: key,
      value,
    }))
  })

  /** 状态分布 */
  const statusDistribution = computed(() => [
    { key: 'online', label: '在线', value: onlineCount.value, color: '#22C55E' },
    { key: 'offline', label: '离线', value: offlineCount.value, color: '#6B6B72' },
    { key: 'error', label: '异常', value: errorCount.value, color: '#EF4444' },
  ])

  return {
    total,
    online,
    offline,
    errorCount,
    unhealthyCount,
    onlineCount,
    offlineCount,
    totalTunnels,
    totalProjects,
    totalConnections,
    totalTraffic,
    totalUploadSpeed,
    totalDownloadSpeed,
    avgPing,
    avgHealth,
    kindDistribution,
    statusDistribution,
  }
}
