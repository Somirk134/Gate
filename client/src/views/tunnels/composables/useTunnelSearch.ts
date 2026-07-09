/* ==================================================================
   useTunnelSearch — 隧道模糊搜索组合式函数
   ------------------------------------------------------------------
   搜索范围：名称 / 协议 / 端口 / 项目 / 服务器 / 标签 / 状态文本
   ================================================================== */

import { computed, type Ref } from 'vue'
import { useI18n } from 'vue-i18n'
import type { Tunnel } from '../types'

export function useTunnelSearch(tunnels: Ref<Tunnel[]>, query: Ref<string>) {
  const { t, te, locale } = useI18n()
  const normalizedQuery = computed(() => query.value.trim().toLowerCase())

  const results = computed(() => {
    void locale.value
    const q = normalizedQuery.value
    if (!q) return tunnels.value
    return tunnels.value.filter((tunnel) => {
      // 名称
      if (tunnel.name.toLowerCase().includes(q)) return true
      // 协议
      if (tunnel.protocol.toLowerCase().includes(q)) return true
      // 本地端口 / 公网端口
      if (String(tunnel.localPort).includes(q)) return true
      if (String(tunnel.remotePort).includes(q)) return true
      // 公网地址
      if (tunnel.publicAddr.toLowerCase().includes(q)) return true
      // 项目
      if (tunnel.projectName.toLowerCase().includes(q)) return true
      // 服务器
      if (tunnel.serverName.toLowerCase().includes(q)) return true
      // 标签
      if (tunnel.tags.some((tag) => tag.toLowerCase().includes(q))) return true
      if (tunnel.tags.some((tag) => localizedTag(tag).toLowerCase().includes(q))) return true
      // 状态文本
      if (t(`tunnel.statusLabels.${tunnel.status}`).toLowerCase().includes(q)) return true
      return false
    })
  })

  function localizedTag(tag: string): string {
    const key = `tunnel.tags.${tag}`
    return te(key) ? t(key) : tag
  }

  const hasQuery = computed(() => normalizedQuery.value.length > 0)
  const matchCount = computed(() => results.value.length)

  return { results, hasQuery, matchCount }
}
