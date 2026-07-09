/* ==================================================================
   useServerSort — 服务器排序组合式函数
   ------------------------------------------------------------------
   支持：名称 / Ping / CPU / Memory / Tunnel 数 / Project 数 / 地区
   收藏服务器永远排最前。
   ================================================================== */

import { computed, ref, type Ref } from 'vue'
import type { Server, ServerSortType, SortDirection } from '../types'

export function useServerSort(
  servers: Ref<Server[]>,
  sortBy: Ref<ServerSortType>,
  direction: Ref<SortDirection> = ref<SortDirection>('asc'),
) {
  const sorted = computed(() => {
    const arr = [...servers.value]
    const dir = direction.value === 'asc' ? 1 : -1

    arr.sort((a, b) => {
      // 收藏永远排前
      if (a.favorite !== b.favorite) return a.favorite ? -1 : 1

      let cmp = 0
      switch (sortBy.value) {
        case 'name':
          cmp = a.name.localeCompare(b.name)
          break
        case 'ping':
          cmp = a.ping - b.ping
          break
        case 'cpu':
          cmp = a.monitor.cpu.percent - b.monitor.cpu.percent
          break
        case 'memory':
          cmp = a.monitor.memory.percent - b.monitor.memory.percent
          break
        case 'tunnels':
          cmp = a.statistics.tunnelCount - b.statistics.tunnelCount
          break
        case 'projects':
          cmp = a.statistics.projectCount - b.statistics.projectCount
          break
        case 'region':
          cmp = a.region.localeCompare(b.region)
          break
        default:
          cmp = 0
      }
      return cmp * dir
    })

    return arr
  })

  return { sorted }
}
