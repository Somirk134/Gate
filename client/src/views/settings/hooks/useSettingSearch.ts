import { computed, watch } from "vue"
import { storeToRefs } from "pinia"
import { useSettingsStore } from "../stores"
import { countSettings, searchSettings } from "../utils"

export function useSettingSearch() {
  const store = useSettingsStore()
  const { categories, searchQuery, categoryFilter, activeCategoryId, selectedSettingId } = storeToRefs(store)

  const visibleCategories = computed(() => {
    const query = searchQuery.value.trim()

    if (query) {
      return searchSettings(categories.value, query, categoryFilter.value)
    }

    const categoryId = categoryFilter.value === "all" ? activeCategoryId.value : categoryFilter.value
    return categories.value.filter((category) => category.id === categoryId)
  })

  const resultCount = computed(() => countSettings(visibleCategories.value))
  const isSearching = computed(() => searchQuery.value.trim().length > 0 || categoryFilter.value !== "all")
  const hasResults = computed(() => resultCount.value > 0)

  watch(
    visibleCategories,
    (nextCategories) => {
      const visibleIds = new Set(
        nextCategories.flatMap((category) => category.groups.flatMap((group) => group.items.map((item) => item.id))),
      )

      if (selectedSettingId.value && visibleIds.has(selectedSettingId.value)) return

      const firstItem = nextCategories[0]?.groups[0]?.items[0]
      if (firstItem) store.setSelectedSetting(firstItem.id)
    },
    { immediate: true },
  )

  return {
    visibleCategories,
    resultCount,
    isSearching,
    hasResults,
    setSearchQuery: store.setSearchQuery,
    setCategoryFilter: store.setCategoryFilter,
  }
}
