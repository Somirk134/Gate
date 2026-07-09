/* ==================================================================
   useTunnelFilter — 隧道筛选组合式函数
   ------------------------------------------------------------------
   支持：全部 / HTTP / TCP / 运行中 / 已停止 / 收藏 / 最近
   ================================================================== */

import { computed, type Ref } from 'vue'
import type { Tunnel, TunnelFilterType } from '../types'
import { isRunningStatus } from '../utils'

export function useTunnelFilter(tunnels: Ref<Tunnel[]>, filter: Ref<TunnelFilterType>) {
  const filtered = computed(() => {
    switch (filter.value) {
      case 'http':
        return tunnels.value.filter((t) => t.protocol === 'http')
      case 'tcp':
        return tunnels.value.filter((t) => t.protocol === 'tcp')
      case 'running':
        return tunnels.value.filter((t) => isRunningStatus(t.status))
      case 'stopped':
        return tunnels.value.filter((t) => t.status === 'stopped' || t.status === 'offline')
      case 'favorite':
        return tunnels.value.filter((t) => t.favorite)
      case 'recent':
        return [...tunnels.value]
          .sort((a, b) => new Date(b.updatedAt).getTime() - new Date(a.updatedAt).getTime())
          .slice(0, 12)
      case 'all':
      default:
        return tunnels.value
    }
  })

  const counts = computed(() => ({
    all: tunnels.value.length,
    http: tunnels.value.filter((t) => t.protocol === 'http').length,
    tcp: tunnels.value.filter((t) => t.protocol === 'tcp').length,
    running: tunnels.value.filter((t) => isRunningStatus(t.status)).length,
    stopped: tunnels.value.filter((t) => t.status === 'stopped' || t.status === 'offline').length,
    favorite: tunnels.value.filter((t) => t.favorite).length,
    recent: Math.min(tunnels.value.length, 12),
  }))

  return { filtered, counts }
}
