/* ==================================================================
   useTunnel — Tunnel 数据加载组合式函数
   ------------------------------------------------------------------
   封装 store 的加载逻辑，提供统一的 loading/error/retry 入口。
   组件层只需调用本 hook，无需直接操作 store 状态机。
   ================================================================== */

import { onMounted } from 'vue'
import { storeToRefs } from 'pinia'
import { useTunnelStore } from '../store/tunnel'
import type { TunnelFormData } from '../types'

export function useTunnel() {
  const store = useTunnelStore()
  const {
    tunnels,
    status,
    error,
    lastUpdated,
    isLoading,
    isError,
    isReady,
    hasTunnels,
    runningTunnels,
    stoppedTunnels,
    httpTunnels,
    tcpTunnels,
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

  function create(form: TunnelFormData) {
    return store.createTunnel(form)
  }

  function update(id: string, patch: Partial<TunnelFormData>) {
    return store.updateTunnel(id, patch)
  }

  function remove(id: string) {
    return store.removeTunnel(id)
  }

  function start(id: string) {
    return store.startTunnel(id)
  }

  function stop(id: string) {
    return store.stopTunnel(id)
  }

  function restart(id: string) {
    return store.restartTunnel(id)
  }

  onMounted(() => {
    if (store.status === 'idle') {
      loadData()
    }
  })

  return {
    // state
    tunnels,
    status,
    error,
    lastUpdated,
    // getters
    isLoading,
    isError,
    isReady,
    hasTunnels,
    runningTunnels,
    stoppedTunnels,
    httpTunnels,
    tcpTunnels,
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
    start,
    stop,
    restart,
    // Store 由 Runtime 轮询组合式函数复用。
    store,
  }
}
