/* ==================================================================
   useProject — Project 数据加载组合式函数
   ------------------------------------------------------------------
   封装 store 的加载逻辑，提供统一的 loading/error/retry 入口。
   组件层只需调用本 hook，无需直接操作 store 状态机。
   ================================================================== */

import { onMounted } from 'vue'
import { storeToRefs } from 'pinia'
import { useProjectStore } from '../store/project'
import type { ProjectFormData } from '../types'

export function useProject() {
  const store = useProjectStore()
  const {
    projects,
    templates,
    status,
    error,
    lastUpdated,
    serverNames,
    isLoading,
    isError,
    isReady,
    hasProjects,
    pinnedProjects,
    favoriteProjects,
    runningProjects,
    stoppedProjects,
    recentProjects,
    totalTunnels,
    runningTunnelCount,
  } = storeToRefs(store)

  async function loadData() {
    await store.load()
  }

  async function refresh() {
    await store.refresh()
  }

  function retry() {
    return store.load()
  }

  function getById(id: string) {
    return store.getById(id)
  }

  function create(form: ProjectFormData) {
    return store.createProject(form)
  }

  function createDefaultProject() {
    return store.createDefaultProject()
  }

  function update(id: string, patch: Partial<ProjectFormData>) {
    return store.updateProject(id, patch)
  }

  function remove(id: string, mode: Parameters<typeof store.removeProject>[1] = 'projectOnly') {
    return store.removeProject(id, mode)
  }

  function start(id: string) {
    return store.startProject(id)
  }

  function stop(id: string) {
    return store.stopProject(id)
  }

  function togglePin(id: string) {
    return store.togglePin(id)
  }

  function toggleFavorite(id: string) {
    return store.toggleFavorite(id)
  }

  onMounted(() => {
    if (store.status === 'idle') {
      loadData()
    } else {
      void refresh()
    }
  })

  return {
    // state
    projects,
    templates,
    status,
    error,
    lastUpdated,
    serverNames,
    // getters
    isLoading,
    isError,
    isReady,
    hasProjects,
    pinnedProjects,
    favoriteProjects,
    runningProjects,
    stoppedProjects,
    recentProjects,
    totalTunnels,
    runningTunnelCount,
    // methods
    getById,
    loadData,
    refresh,
    retry,
    create,
    createDefaultProject,
    update,
    remove,
    start,
    stop,
    togglePin,
    toggleFavorite,
  }
}
