/* ==================================================================
   useProjectFilter — 项目筛选组合式函数
   ------------------------------------------------------------------
   支持：全部 / 运行中 / 已停止 / 收藏 / 最近使用
   ================================================================== */

import { computed, type Ref } from 'vue'
import type { Project, ProjectFilterType } from '../types'

export function useProjectFilter(projects: Ref<Project[]>, filter: Ref<ProjectFilterType>) {
  const filtered = computed(() => {
    switch (filter.value) {
      case 'running':
        return projects.value.filter(
          (p) => p.status === 'running' || p.status === 'partial' || p.status === 'starting',
        )
      case 'stopped':
        return projects.value.filter((p) => p.status === 'stopped')
      case 'favorite':
        return projects.value.filter((p) => p.favorite)
      case 'recent':
        return [...projects.value].sort((a, b) => b.lastUsedAt - a.lastUsedAt).slice(0, 12)
      case 'all':
      default:
        return projects.value
    }
  })

  const counts = computed(() => ({
    all: projects.value.length,
    running: projects.value.filter(
      (p) => p.status === 'running' || p.status === 'partial' || p.status === 'starting',
    ).length,
    stopped: projects.value.filter((p) => p.status === 'stopped').length,
    favorite: projects.value.filter((p) => p.favorite).length,
    recent: Math.min(projects.value.length, 12),
  }))

  return { filtered, counts }
}
