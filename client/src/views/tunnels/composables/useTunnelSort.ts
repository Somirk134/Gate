/* ==================================================================
   useTunnelSort — 隧道排序组合式函数
   ------------------------------------------------------------------
   支持：名称 / 状态 / 流量 / 连接数 / 创建时间 / 更新时间
   固定隧道永远排最前。
   ================================================================== */

import { computed, ref, type Ref } from 'vue'
import type { Tunnel, TunnelSortType, SortDirection } from '../types'
import { STATUS_WEIGHT, totalTraffic } from '../utils'

export function useTunnelSort(
  tunnels: Ref<Tunnel[]>,
  sortBy: Ref<TunnelSortType>,
  direction: Ref<SortDirection> = ref<SortDirection>('asc'),
) {
  const sorted = computed(() => {
    const arr = [...tunnels.value]
    const dir = direction.value === 'asc' ? 1 : -1

    arr.sort((a, b) => {
      // 固定隧道永远排前
      if (a.pinned !== b.pinned) return a.pinned ? -1 : 1

      let cmp = 0
      switch (sortBy.value) {
        case 'name':
          cmp = a.name.localeCompare(b.name)
          break
        case 'status':
          cmp = (STATUS_WEIGHT[a.status] ?? 9) - (STATUS_WEIGHT[b.status] ?? 9)
          break
        case 'traffic':
          cmp = totalTraffic(a) - totalTraffic(b)
          break
        case 'connections':
          cmp = a.statistics.connections - b.statistics.connections
          break
        case 'createdAt':
          cmp = new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime()
          break
        case 'updatedAt':
          cmp = new Date(a.updatedAt).getTime() - new Date(b.updatedAt).getTime()
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
