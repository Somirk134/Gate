/* ==================================================================
   useProjectSearch — 项目模糊搜索组合式函数
   ------------------------------------------------------------------
   搜索范围：名称 / 标签 / Domain / Certificate / 模板 / 状态文本
   ================================================================== */

import { computed, type Ref } from 'vue'
import type { Project } from '../types'
import { STATUS_CONFIG } from '../utils'

export function useProjectSearch(projects: Ref<Project[]>, query: Ref<string>) {
  const normalizedQuery = computed(() => query.value.trim().toLowerCase())

  const results = computed(() => {
    const q = normalizedQuery.value
    if (!q) return projects.value
    return projects.value.filter((p) => {
      // 名称
      if (p.name.toLowerCase().includes(q)) return true
      // 描述
      if (p.description.toLowerCase().includes(q)) return true
      // 标签
      if (p.tags.some((t) => t.toLowerCase().includes(q))) return true
      if (p.domains.some((domain) => domain.toLowerCase().includes(q))) return true
      if (p.certificateIds.some((certificate) => certificate.toLowerCase().includes(q))) return true
      if (p.template.toLowerCase().includes(q)) return true
      // 状态文本
      if (STATUS_CONFIG[p.status].label.toLowerCase().includes(q)) return true
      return false
    })
  })

  const hasQuery = computed(() => normalizedQuery.value.length > 0)
  const matchCount = computed(() => results.value.length)

  return { results, hasQuery, matchCount }
}
