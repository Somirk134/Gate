/* ==================================================================
   useProjectFavorite — 收藏与固定操作组合式函数
   ------------------------------------------------------------------
   封装 toggle 逻辑并提供反馈回调注入点。
   Dashboard 收藏项目自动同步。
   ================================================================== */

import { computed, type Ref } from "vue"
import type { Project } from "../types"

export function useProjectFavorite(projects: Ref<Project[]>) {
  const favoriteList = computed(() =>
    projects.value.filter((p) => p.favorite),
  )

  const pinnedList = computed(() =>
    projects.value.filter((p) => p.pinned),
  )

  function isFavorite(id: string): boolean {
    return projects.value.some((p) => p.id === id && p.favorite)
  }

  function isPinned(id: string): boolean {
    return projects.value.some((p) => p.id === id && p.pinned)
  }

  return {
    favoriteList,
    pinnedList,
    isFavorite,
    isPinned,
  }
}
