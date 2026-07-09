/* ==================================================================
   useServerFilter — 服务器筛选组合式函数
   ------------------------------------------------------------------
   支持：全部 / 在线 / 离线 / 收藏 / 最近 / 健康异常
   ================================================================== */

import { computed, type Ref } from 'vue'
import type { Server, ServerFilterType } from '../types'
import { isOnlineStatus } from '../utils'

export function useServerFilter(servers: Ref<Server[]>, filter: Ref<ServerFilterType>) {
  const filtered = computed(() => {
    switch (filter.value) {
      case 'online':
        return servers.value.filter((s) => isOnlineStatus(s.status))
      case 'offline':
        return servers.value.filter((s) => s.status === 'offline' || s.status === 'disconnected')
      case 'favorite':
        return servers.value.filter((s) => s.favorite)
      case 'recent':
        return servers.value.filter((s) => s.recent)
      case 'unhealthy':
        return servers.value.filter((s) => s.health.overall !== 'healthy')
      case 'all':
      default:
        return servers.value
    }
  })

  const counts = computed(() => ({
    all: servers.value.length,
    online: servers.value.filter((s) => isOnlineStatus(s.status)).length,
    offline: servers.value.filter((s) => s.status === 'offline' || s.status === 'disconnected')
      .length,
    favorite: servers.value.filter((s) => s.favorite).length,
    recent: servers.value.filter((s) => s.recent).length,
    unhealthy: servers.value.filter((s) => s.health.overall !== 'healthy').length,
  }))

  return { filtered, counts }
}
