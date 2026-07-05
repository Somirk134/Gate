/* ==================================================================
   useTunnelSearch — 隧道模糊搜索组合式函数
   ------------------------------------------------------------------
   搜索范围：名称 / 协议 / 端口 / 项目 / 服务器 / 标签 / 状态文本
   ================================================================== */

import { computed, type Ref } from "vue"
import type { Tunnel } from "../types"
import { TUNNEL_STATUS_CONFIG } from "../utils"

export function useTunnelSearch(
  tunnels: Ref<Tunnel[]>,
  query: Ref<string>,
) {
  const normalizedQuery = computed(() => query.value.trim().toLowerCase())

  const results = computed(() => {
    const q = normalizedQuery.value
    if (!q) return tunnels.value
    return tunnels.value.filter((t) => {
      // 名称
      if (t.name.toLowerCase().includes(q)) return true
      // 协议
      if (t.protocol.toLowerCase().includes(q)) return true
      // 本地端口 / 公网端口
      if (String(t.localPort).includes(q)) return true
      if (String(t.remotePort).includes(q)) return true
      // 公网地址
      if (t.publicAddr.toLowerCase().includes(q)) return true
      // 项目
      if (t.projectName.toLowerCase().includes(q)) return true
      // 服务器
      if (t.serverName.toLowerCase().includes(q)) return true
      // 标签
      if (t.tags.some((tag) => tag.toLowerCase().includes(q))) return true
      // 状态文本
      if (TUNNEL_STATUS_CONFIG[t.status].label.toLowerCase().includes(q)) return true
      return false
    })
  })

  const hasQuery = computed(() => normalizedQuery.value.length > 0)
  const matchCount = computed(() => results.value.length)

  return { results, hasQuery, matchCount }
}
