<template>
  <div class="settings-page">
    <SettingsHeader
      ref="headerRef"
      :categories="categories"
      :query="searchQuery"
      :category-filter="categoryFilter"
      :result-count="resultCount"
      :modified-count="modifiedCount"
      @update:query="setSearchQuery"
      @update:category-filter="handleCategoryFilter"
      @reset-all="resetAll"
    />

    <div class="settings-workspace">
      <SettingsSidebar
        :categories="categories"
        :active-category-id="activeCategoryId"
        :active-group-id="activeGroupId"
        @select-category="handleSelectCategory"
        @select-group="handleSelectGroup"
      />

      <main ref="contentRef" class="settings-content">
        <SettingsLoading v-if="loading" />
        <SettingsEmpty v-else-if="!hasResults" :query="searchQuery" />

        <template v-else>
          <SettingsCategory
            v-for="category in visibleCategories"
            :key="category.id"
            :category="category"
            :values="values"
            :validation-errors="validationErrors"
            :dirty-keys="dirtyKeys"
            :selected-setting-id="selectedSettingId"
            :highlight-query="searchQuery"
            :action-statuses="actionStatuses"
            @select="setSelectedSetting($event.id)"
            @reset="resetSetting"
            @run-action="runMockAction"
            @update:value="setValue"
          />
        </template>
      </main>

      <SettingsInspector
        :context="selectedContext"
        :values="values"
        :dirty-keys="dirtyKeys"
        :validation-errors="validationErrors"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { nextTick, ref } from "vue"
import type { SettingCategoryId } from "./types"
import SettingsCategory from "./components/SettingsCategory.vue"
import SettingsEmpty from "./components/SettingsEmpty.vue"
import SettingsHeader from "./components/SettingsHeader.vue"
import SettingsInspector from "./components/SettingsInspector.vue"
import SettingsLoading from "./components/SettingsLoading.vue"
import SettingsSidebar from "./components/SettingsSidebar.vue"
import { useLocaleSetting, useSettingReset, useSettingSearch, useSettings, useShortcut, useTheme } from "./hooks"
import "./styles/settings.css"

const {
  categories,
  values,
  validationErrors,
  activeCategoryId,
  activeGroupId,
  selectedSettingId,
  selectedContext,
  searchQuery,
  categoryFilter,
  loading,
  actionStatuses,
  setValue,
  resetSetting,
  resetAll,
  setActiveCategory,
  setActiveGroup,
  setSelectedSetting,
  setSearchQuery,
  setCategoryFilter,
  runMockAction,
} = useSettings()
const { visibleCategories, resultCount, hasResults } = useSettingSearch()
const { dirtyKeys, modifiedCount } = useSettingReset()
const headerRef = ref<InstanceType<typeof SettingsHeader> | null>(null)
const contentRef = ref<HTMLElement | null>(null)

useTheme()
useLocaleSetting()
useShortcut({
  onSearch: () => headerRef.value?.focusSearch(),
})

function handleCategoryFilter(categoryId: SettingCategoryId | "all") {
  setCategoryFilter(categoryId)
  if (categoryId !== "all") setActiveCategory(categoryId)
  scrollContentToTop()
}

function handleSelectCategory(categoryId: SettingCategoryId) {
  setCategoryFilter("all")
  setActiveCategory(categoryId)
  scrollToElement(`settings-category-${categoryId}`)
}

function handleSelectGroup(groupId: string) {
  setCategoryFilter("all")
  setActiveGroup(groupId)
  scrollToElement(`settings-group-${groupId}`)
}

function scrollContentToTop() {
  nextTick(() => {
    contentRef.value?.scrollTo({ top: 0, behavior: "smooth" })
  })
}

function scrollToElement(id: string) {
  nextTick(() => {
    document.getElementById(id)?.scrollIntoView({ block: "start", behavior: "smooth" })
  })
}
</script>
