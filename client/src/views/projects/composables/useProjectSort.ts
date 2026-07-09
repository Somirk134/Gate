/* ==================================================================
   useProjectSort — 项目排序组合式函数
   ------------------------------------------------------------------
   支持：名称 / 创建时间 / 更新时间 / 运行状态 / Tunnel 数量
   固定项目永远排最前。
   ================================================================== */

import { computed, ref, type Ref } from 'vue'
import type { Project, ProjectSortType, SortDirection } from '../types'

const STATUS_WEIGHT: Record<string, number> = {
  running: 0,
  starting: 1,
  partial: 2,
  error: 3,
  stopped: 4,
}

export function useProjectSort(
  projects: Ref<Project[]>,
  sortBy: Ref<ProjectSortType>,
  direction: Ref<SortDirection> = ref<SortDirection>('asc'),
) {
  const sorted = computed(() => {
    const arr = [...projects.value]
    const dir = direction.value === 'asc' ? 1 : -1

    arr.sort((a, b) => {
      // 固定项目永远排前
      if (a.pinned !== b.pinned) return a.pinned ? -1 : 1

      let cmp = 0
      switch (sortBy.value) {
        case 'name':
          cmp = a.name.localeCompare(b.name)
          break
        case 'createdAt':
          cmp = new Date(a.createdAt).getTime() - new Date(b.createdAt).getTime()
          break
        case 'updatedAt':
          cmp = new Date(a.updatedAt).getTime() - new Date(b.updatedAt).getTime()
          break
        case 'status':
          cmp = (STATUS_WEIGHT[a.status] ?? 9) - (STATUS_WEIGHT[b.status] ?? 9)
          break
        case 'tunnelCount':
          cmp = a.tunnelCount - b.tunnelCount
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

/* 状态排序权重导出（供其他地方复用） */
export { STATUS_WEIGHT }
