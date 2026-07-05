/* ==================================================================
   useTunnel — Tunnel 数据加载组合式函数
   ------------------------------------------------------------------
   封装 store 的加载逻辑，提供统一的 loading/error/retry 入口。
   组件层只需调用本 hook，无需直接操作 store 状态机。
   ================================================================== */

import { onMounted } from "vue"
import { storeToRefs } from "pinia"
import { useTunnelStore } from "../store/tunnel"
import type { TunnelFormData } from "../types"

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
    pinnedTunnels,
    favoriteTunnels,
    runningTunnels,
    stoppedTunnels,
    recentTunnels,
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
    store.updateTunnel(id, patch)
  }

  function remove(id: string) {
    store.removeTunnel(id)
  }

  function start(id: string) {
    store.startTunnel(id)
  }

  function stop(id: string) {
    store.stopTunnel(id)
  }

  function restart(id: string) {
    store.restartTunnel(id)
  }

  function clone(id: string) {
    return store.cloneTunnel(id)
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
    tunnels,
    status,
    error,
    lastUpdated,
    // getters
    isLoading,
    isError,
    isReady,
    hasTunnels,
    pinnedTunnels,
    favoriteTunnels,
    runningTunnels,
    stoppedTunnels,
    recentTunnels,
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
    clone,
    togglePin,
    toggleFavorite,
    // store（供 monitor 等 hook 直接调用 tick）
    store,
  }
}
