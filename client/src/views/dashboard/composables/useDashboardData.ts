/* ==================================================================
   useDashboardData — Dashboard 数据加载组合式函数
   ------------------------------------------------------------------
   封装 store 的加载逻辑，提供统一的 loading/error/retry 入口。
   组件层只需调用本 hook，无需直接操作 store 状态机。
   ================================================================== */

import { onMounted } from "vue"
import { storeToRefs } from "pinia"
import { useDashboardStore } from "../store/dashboard"

export function useDashboardData() {
  const store = useDashboardStore()
  const {
    projects,
    tunnels,
    servers,
    activities,
    statistics,
    resource,
    news,
    actions,
    status,
    error,
    isLoading,
    isError,
    isReady,
    hasProjects,
    randomQuote,
    lastUpdated,
    runningTunnels,
    onlineServers,
    connectedServerCount,
    totalConnections,
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

  onMounted(() => {
    if (store.status === "idle") {
      loadData()
    }
  })

  return {
    // state
    projects,
    tunnels,
    servers,
    activities,
    statistics,
    resource,
    news,
    actions,
    status,
    error,
    lastUpdated,
    // getters
    isLoading,
    isError,
    isReady,
    hasProjects,
    randomQuote,
    runningTunnels,
    onlineServers,
    connectedServerCount,
    totalConnections,
    // actions
    loadData,
    refresh,
    retry,
    togglePin: store.togglePin,
    toggleFavorite: store.toggleFavorite,
    startTunnel: store.startTunnel,
    stopTunnel: store.stopTunnel,
  }
}
