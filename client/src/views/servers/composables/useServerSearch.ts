/* ==================================================================
   useServerSearch — 服务器模糊搜索组合式函数
   ------------------------------------------------------------------
   搜索范围：名称 / 地区 / IP / 版本 / 类型 / 标签 / 状态文本
   ================================================================== */

import { computed, type Ref } from 'vue'
import { useI18n } from 'vue-i18n'
import type { Server } from '../types'
import { KIND_MAP } from '../utils'

export function useServerSearch(servers: Ref<Server[]>, query: Ref<string>) {
  const { t, te, locale } = useI18n()
  const normalizedQuery = computed(() => query.value.trim().toLowerCase())

  const results = computed(() => {
    void locale.value
    const q = normalizedQuery.value
    if (!q) return servers.value
    return servers.value.filter((s) => {
      // 名称
      if (s.name.toLowerCase().includes(q)) return true
      // 地区
      if (s.region.toLowerCase().includes(q)) return true
      // 公网 IP
      if (s.publicIp.toLowerCase().includes(q)) return true
      // 版本
      if (s.version.toLowerCase().includes(q)) return true
      // 类型
      if (t(`server.kinds.${KIND_MAP[s.kind]?.label}.label`).toLowerCase().includes(q)) return true
      // 主机
      if (s.settings.host.toLowerCase().includes(q)) return true
      // 标签
      if (s.tags.some((tag) => tag.toLowerCase().includes(q))) return true
      if (s.tags.some((tag) => localizedTag(tag).toLowerCase().includes(q))) return true
      // 状态文本
      if (t(`server.statusLabels.${s.status}`).toLowerCase().includes(q)) return true
      return false
    })
  })

  function localizedTag(tag: string): string {
    const key = `server.tags.${tag}`
    return te(key) ? t(key) : tag
  }

  const hasQuery = computed(() => normalizedQuery.value.length > 0)
  const matchCount = computed(() => results.value.length)

  return { results, hasQuery, matchCount }
}
