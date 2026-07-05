/* ==================================================================
   useProject — Project 数据加载组合式函数
   ------------------------------------------------------------------
   封装 store 的加载逻辑，提供统一的 loading/error/retry 入口。
   组件层只需调用本 hook，无需直接操作 store 状态机。
   ================================================================== */

import { onMounted } from "vue"
import { storeToRefs } from "pinia"
import { useProjectStore } from "../store/project"
import type { ProjectFormData } from "../types"

export function useProject() {
  const store = useProjectStore()
  const {
    projects,
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

  function update(id: string, patch: Partial<ProjectFormData>) {
    store.updateProject(id, patch)
  }

  function remove(id: string) {
    store.removeProject(id)
  }

  function start(id: string) {
    store.startProject(id)
  }

  function stop(id: string) {
    store.stopProject(id)
  }

  function togglePin(id: string) {
    store.togglePin(id)
  }

  function toggleFavorite(id: string) {
    store.toggleFavorite(id)
  }

  onMounted(() => {
    if (store.status === "idle") {
      loadData()
    }
  })

  return {
    // state
    projects,
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
    update,
    remove,
    start,
    stop,
    togglePin,
    toggleFavorite,
  }
}
