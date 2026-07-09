/* ==================================================================
   useTunnelStatistics — 隧道统计聚合组合式函数
   ------------------------------------------------------------------
   从隧道集合派生汇总统计，供工具栏 / 概览 / Dashboard 复用。
   ================================================================== */

import { computed, type Ref } from 'vue'
import type { Tunnel } from '../types'
import { isRunningStatus } from '../utils'

export function useTunnelStatistics(tunnels: Ref<Tunnel[]>) {
  const total = computed(() => tunnels.value.length)
  const running = computed(() => tunnels.value.filter((t) => isRunningStatus(t.status)))
  const stopped = computed(() =>
    tunnels.value.filter((t) => t.status === 'stopped' || t.status === 'offline'),
  )
  const errorCount = computed(() => tunnels.value.filter((t) => t.status === 'error').length)

  const httpCount = computed(() => tunnels.value.filter((t) => t.protocol === 'http').length)
  const tcpCount = computed(() => tunnels.value.filter((t) => t.protocol === 'tcp').length)

  const runningCount = computed(() => running.value.length)
  const stoppedCount = computed(() => stopped.value.length)

  const totalConnections = computed(() =>
    tunnels.value.reduce((sum, t) => sum + t.statistics.connections, 0),
  )
  const totalRequests = computed(() =>
    tunnels.value.reduce((sum, t) => sum + t.statistics.requests, 0),
  )
  const totalTraffic = computed(() =>
    tunnels.value.reduce((sum, t) => sum + t.traffic.totalUpload + t.traffic.totalDownload, 0),
  )
  const totalUploadSpeed = computed(() =>
    tunnels.value.reduce((sum, t) => sum + t.traffic.uploadSpeed, 0),
  )
  const totalDownloadSpeed = computed(() =>
    tunnels.value.reduce((sum, t) => sum + t.traffic.downloadSpeed, 0),
  )
  const avgLatency = computed(() => {
    const r = running.value
    if (r.length === 0) return 0
    return Math.round(r.reduce((sum, t) => sum + t.statistics.avgLatency, 0) / r.length)
  })

  /** 协议分布 */
  const protocolDistribution = computed(() => [
    { key: 'http', label: 'HTTP', value: httpCount.value, color: '#5B8DEF' },
    { key: 'tcp', label: 'TCP', value: tcpCount.value, color: '#22C55E' },
  ])

  /** 状态分布 */
  const statusDistribution = computed(() => [
    { key: 'running', label: '运行中', value: runningCount.value, color: '#22C55E' },
    { key: 'stopped', label: '已停止', value: stoppedCount.value, color: '#6B6B72' },
    { key: 'error', label: '异常', value: errorCount.value, color: '#EF4444' },
  ])

  return {
    total,
    running,
    stopped,
    errorCount,
    httpCount,
    tcpCount,
    runningCount,
    stoppedCount,
    totalConnections,
    totalRequests,
    totalTraffic,
    totalUploadSpeed,
    totalDownloadSpeed,
    avgLatency,
    protocolDistribution,
    statusDistribution,
  }
}
