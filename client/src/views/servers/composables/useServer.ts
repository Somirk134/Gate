/* ==================================================================
   useServer — Server 数据加载组合式函数
   ------------------------------------------------------------------
   封装 store 的加载逻辑，提供统一的 loading/error/retry 入口。
   组件层只需调用本 hook，无需直接操作 store 状态机。
   ================================================================== */

import { onMounted } from 'vue'
import { storeToRefs } from 'pinia'
import { useServerStore } from '../store/server'
import type { ServerFormData } from '../types'

export function useServer() {
  const store = useServerStore()
  const {
    servers,
    status,
    error,
    lastUpdated,
    isLoading,
    isError,
    isReady,
    hasServers,
    onlineServers,
    offlineServers,
    errorServers,
    favoriteServers,
    recentServers,
    unhealthyServers,
    totalTunnels,
    totalProjects,
    totalConnections,
    totalTraffic,
    totalUploadSpeed,
    totalDownloadSpeed,
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

  function create(form: ServerFormData) {
    return store.createServer(form)
  }

  function update(id: string, patch: Partial<ServerFormData>) {
    return store.updateServer(id, patch)
  }

  function remove(id: string) {
    return store.removeServer(id)
  }

  function connect(id: string) {
    return store.connectServer(id)
  }

  function disconnect(id: string) {
    return store.disconnectServer(id)
  }

  function restart(id: string) {
    return store.restartServer(id)
  }

  function checkHealth(id: string) {
    return store.checkHealth(id)
  }

  function toggleFavorite(id: string) {
    store.toggleFavorite(id)
  }

  onMounted(() => {
    if (store.status === 'idle') {
      loadData()
    }
  })

  return {
    // state
    servers,
    status,
    error,
    lastUpdated,
    // getters
    isLoading,
    isError,
    isReady,
    hasServers,
    onlineServers,
    offlineServers,
    errorServers,
    favoriteServers,
    recentServers,
    unhealthyServers,
    totalTunnels,
    totalProjects,
    totalConnections,
    totalTraffic,
    totalUploadSpeed,
    totalDownloadSpeed,
    // methods
    getById,
    loadData,
    refresh,
    retry,
    create,
    update,
    remove,
    connect,
    disconnect,
    restart,
    checkHealth,
    toggleFavorite,
    // store（供 monitor hook 直接调用 tick）
    store,
  }
}
