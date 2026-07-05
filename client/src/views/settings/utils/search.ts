import type {
  SettingCategory,
  SettingCategoryId,
  SettingGroup,
  SettingItem,
  SettingSearchMatch,
  SettingSearchResult,
} from "../types"

export interface HighlightPart {
  text: string
  highlighted: boolean
}

export function searchSettings(
  categories: SettingCategory[],
  query: string,
  categoryFilter: SettingCategoryId | "all",
) {
  const normalizedQuery = normalizeSearchText(query)
  const scopedCategories = categoryFilter === "all"
    ? categories
    : categories.filter((category) => category.id === categoryFilter)

  if (!normalizedQuery) return scopedCategories

  const grouped = new Map<string, SettingCategory>()

  for (const category of scopedCategories) {
    const groups: SettingGroup[] = []

    for (const group of category.groups) {
      const scoredItems = group.items
        .map((item) => scoreSettingItem(category, group, item, normalizedQuery))
        .filter((result): result is SettingSearchResult => !!result)
        .sort((a, b) => b.score - a.score)
        .map((result) => result.item)

      if (scoredItems.length) {
        groups.push({ ...group, items: scoredItems })
      }
    }

    if (groups.length) {
      grouped.set(category.id, { ...category, groups })
    }
  }

  return [...grouped.values()].sort((a, b) => a.order - b.order)
}

export function countSettings(categories: SettingCategory[]) {
  return categories.reduce(
    (total, category) => total + category.groups.reduce((groupTotal, group) => groupTotal + group.items.length, 0),
    0,
  )
}

export function highlightText(text: string, query: string): HighlightPart[] {
  const terms = getSearchTerms(query)
  if (!terms.length || !text) return [{ text, highlighted: false }]

  const ranges: Array<[number, number]> = []
  const lowerText = text.toLowerCase()

  for (const term of terms) {
    let index = lowerText.indexOf(term)
    while (index !== -1) {
      ranges.push([index, index + term.length])
      index = lowerText.indexOf(term, index + term.length)
    }
  }

  if (!ranges.length) return [{ text, highlighted: false }]

  ranges.sort((a, b) => a[0] - b[0])
  const merged = ranges.reduce<Array<[number, number]>>((result, range) => {
    const last = result[result.length - 1]
    if (!last || range[0] > last[1]) {
      result.push([...range])
      return result
    }

    last[1] = Math.max(last[1], range[1])
    return result
  }, [])

  const parts: HighlightPart[] = []
  let cursor = 0

  for (const [start, end] of merged) {
    if (start > cursor) parts.push({ text: text.slice(cursor, start), highlighted: false })
    parts.push({ text: text.slice(start, end), highlighted: true })
    cursor = end
  }

  if (cursor < text.length) parts.push({ text: text.slice(cursor), highlighted: false })

  return parts
}

export function normalizeSearchText(value: string) {
  return value.trim().toLowerCase()
}

function scoreSettingItem(
  category: SettingCategory,
  group: SettingGroup,
  item: SettingItem,
  normalizedQuery: string,
): SettingSearchResult | null {
  const fields: Array<[SettingSearchMatch["field"], string, number]> = [
    ["label", item.label, 64],
    ["key", item.key, 54],
    ["description", item.description, 34],
    ["tags", item.tags?.join(" ") ?? "", 28],
    ["category", category.label, 22],
    ["group", group.label, 18],
  ]

  const matches: SettingSearchMatch[] = []
  let score = 0

  for (const [field, value, weight] of fields) {
    const fieldScore = scoreText(value, normalizedQuery)
    if (fieldScore > 0) {
      const weightedScore = fieldScore + weight
      score += weightedScore
      matches.push({ field, score: weightedScore })
    }
  }

  if (!matches.length) return null

  return { category, group, item, score, matches }
}

function scoreText(value: string, normalizedQuery: string) {
  const normalizedValue = normalizeSearchText(value)
  if (!normalizedValue || !normalizedQuery) return 0

  const terms = getSearchTerms(normalizedQuery)
  let score = 0

  for (const term of terms) {
    if (normalizedValue === term) {
      score += 120
      continue
    }

    if (normalizedValue.includes(term)) {
      score += 80 + term.length
      continue
    }

    const fuzzyScore = scoreFuzzyMatch(term, normalizedValue)
    if (fuzzyScore > 0) {
      score += fuzzyScore
    }
  }

  return score
}

function getSearchTerms(value: string) {
  return normalizeSearchText(value).split(/\s+/).filter(Boolean)
}

function scoreFuzzyMatch(needle: string, haystack: string) {
  if (needle.length < 3) return 0

  let needleIndex = 0
  let firstIndex = -1
  let lastIndex = -1

  for (let haystackIndex = 0; haystackIndex < haystack.length; haystackIndex += 1) {
    if (haystack[haystackIndex] !== needle[needleIndex]) continue

    if (firstIndex === -1) firstIndex = haystackIndex
    lastIndex = haystackIndex
    needleIndex += 1

    if (needleIndex === needle.length) {
      const span = lastIndex - firstIndex + 1
      const maxSpan = Math.max(needle.length * 2 + 2, 8)
      if (span > maxSpan) return 0
      return Math.max(12, 44 - span)
    }
  }

  return 0
}
