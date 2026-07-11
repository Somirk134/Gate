/* ==================================================================
   useProjectSearch — 项目模糊搜索组合式函数
   ------------------------------------------------------------------
   搜索范围：名称 / 标签 / Domain / Certificate / 模板 / 状态文本
   ================================================================== */

import { computed, type Ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { translateIfExists } from '@/utils/i18n'
import type { Project } from '../types'

export function useProjectSearch(projects: Ref<Project[]>, query: Ref<string>) {
  const { t, te, locale } = useI18n()
  const normalizedQuery = computed(() => query.value.trim().toLowerCase())

  const results = computed(() => {
    // 显式依赖当前语言，确保切换语言后搜索状态和内置标签实时更新。
    void locale.value
    const q = normalizedQuery.value
    if (!q) return projects.value
    return projects.value.filter((p) => {
      // 名称
      if (p.name.toLowerCase().includes(q)) return true
      // 描述
      if (p.description.toLowerCase().includes(q)) return true
      // 标签
      if (p.tags.some((t) => t.toLowerCase().includes(q))) return true
      if (p.tags.some((tag) => localizedTag(tag).toLowerCase().includes(q))) return true
      if (p.domains.some((domain) => domain.toLowerCase().includes(q))) return true
      if (p.certificateIds.some((certificate) => certificate.toLowerCase().includes(q))) return true
      if (p.template.toLowerCase().includes(q)) return true
      // 状态文本
      if (t(`project.statusLabels.${p.status}`).toLowerCase().includes(q)) return true
      return false
    })
  })

  function localizedTag(tag: string): string {
    const key = `project.tags.${tag}`
    return translateIfExists(t, te, key, tag)
  }

  const hasQuery = computed(() => normalizedQuery.value.length > 0)
  const matchCount = computed(() => results.value.length)

  return { results, hasQuery, matchCount }
}
