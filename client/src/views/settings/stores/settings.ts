import { defineStore } from 'pinia'
import { computed, ref, watch } from 'vue'
import type {
  SettingActionStatus,
  SettingCategory,
  SettingCategoryId,
  SettingItem,
  SettingValue,
} from '../types'
import {
  cloneSettingValue,
  findSettingContext,
  flattenSettingItems,
  getCategoryById,
  getDefaultValues,
  isEqualSettingValue,
  validateSettingValue,
} from '../utils'

const settingsCategories: SettingCategory[] = [
  {
    id: 'general',
    label: '通用',
    description: '基础应用偏好。',
    icon: 'settings',
    order: 1,
    groups: [
      {
        id: 'general.language',
        categoryId: 'general',
        label: '语言',
        items: [
          {
            id: 'general.language',
            key: 'general.language',
            categoryId: 'general',
            groupId: 'general.language',
            label: '语言',
            description: '界面语言。',
            control: {
              type: 'select',
              options: [
                { label: '简体中文', value: 'zh-CN' },
                { label: 'English', value: 'en-US' },
              ],
            },
            defaultValue: 'zh-CN',
            status: 'stable',
          },
        ],
      },
    ],
  },
  {
    id: 'appearance',
    label: '外观',
    description: '界面主题偏好。',
    icon: 'palette',
    order: 2,
    groups: [
      {
        id: 'appearance.theme',
        categoryId: 'appearance',
        label: '主题',
        items: [
          {
            id: 'appearance.theme',
            key: 'appearance.theme',
            categoryId: 'appearance',
            groupId: 'appearance.theme',
            label: '主题',
            description: '选择浅色、深色或跟随系统。',
            control: {
              type: 'select',
              options: [
                { label: '深色', value: 'dark' },
                { label: '浅色', value: 'light' },
                { label: '跟随系统', value: 'system' },
              ],
            },
            defaultValue: 'dark',
            status: 'stable',
          },
        ],
      },
    ],
  },
]

export const useSettingsStore = defineStore('settings', () => {
  const categories = ref<SettingCategory[]>(settingsCategories)
  const defaults = computed(() => getDefaultValues(categories.value))
  const values = ref<Record<string, SettingValue>>(getDefaultValues(settingsCategories))
  const validationErrors = ref<Record<string, string | undefined>>({})
  const activeCategoryId = ref<SettingCategoryId>('general')
  const activeGroupId = ref<string | null>(null)
  const selectedSettingId = ref<string | null>('general.language')
  const searchQuery = ref('')
  const categoryFilter = ref<SettingCategoryId | 'all'>('all')
  const loading = ref(false)
  const actionStatuses = ref<Record<string, SettingActionStatus>>({})

  const allContexts = computed(() => flattenSettingItems(categories.value))
  const allItems = computed(() => allContexts.value.map(({ item }) => item))
  const currentCategory = computed(() => getCategoryById(categories.value, activeCategoryId.value))
  const selectedContext = computed(() => {
    return (
      findSettingContext(categories.value, selectedSettingId.value) ?? allContexts.value[0] ?? null
    )
  })
  const selectedSetting = computed(() => selectedContext.value?.item ?? null)
  const dirtyKeys = computed(() =>
    Object.keys(defaults.value).filter(
      (key) => !isEqualSettingValue(values.value[key], defaults.value[key]),
    ),
  )
  const modifiedCount = computed(() => dirtyKeys.value.length)
  const hasModifiedSettings = computed(() => modifiedCount.value > 0)

  function read<T extends SettingValue = SettingValue>(key: string) {
    return values.value[key] as T | undefined
  }

  function getValue(itemOrKey: SettingItem | string) {
    const key = typeof itemOrKey === 'string' ? itemOrKey : itemOrKey.key
    return values.value[key]
  }

  function setValue(itemOrKey: SettingItem | string, value: SettingValue) {
    const context = findSettingContext(
      categories.value,
      typeof itemOrKey === 'string' ? itemOrKey : itemOrKey.key,
    )
    if (!context) return false

    const error = validateSettingValue(context.item, value)
    validationErrors.value = {
      ...validationErrors.value,
      [context.item.key]: error,
    }

    if (error) return false

    values.value = {
      ...values.value,
      [context.item.key]: cloneSettingValue(value),
    }
    selectedSettingId.value = context.item.id
    return true
  }

  function resetSetting(itemOrKey: SettingItem | string) {
    const context = findSettingContext(
      categories.value,
      typeof itemOrKey === 'string' ? itemOrKey : itemOrKey.key,
    )
    if (!context) return

    values.value = {
      ...values.value,
      [context.item.key]: cloneSettingValue(context.item.defaultValue),
    }
    validationErrors.value = {
      ...validationErrors.value,
      [context.item.key]: undefined,
    }
    selectedSettingId.value = context.item.id
  }

  function resetCategory(categoryId: SettingCategoryId) {
    const category = getCategoryById(categories.value, categoryId)
    const nextValues = { ...values.value }
    const nextErrors = { ...validationErrors.value }

    for (const group of category.groups) {
      for (const item of group.items) {
        nextValues[item.key] = cloneSettingValue(item.defaultValue)
        nextErrors[item.key] = undefined
      }
    }

    values.value = nextValues
    validationErrors.value = nextErrors
  }

  function resetAll() {
    values.value = getDefaultValues(categories.value)
    validationErrors.value = {}
  }

  function importSettings(payload: Record<string, SettingValue>) {
    const nextValues = { ...values.value }
    const nextErrors = { ...validationErrors.value }

    for (const [key, value] of Object.entries(payload)) {
      const context = findSettingContext(categories.value, key)
      if (!context) continue

      const error = validateSettingValue(context.item, value)
      nextErrors[key] = error
      if (!error) nextValues[key] = cloneSettingValue(value)
    }

    values.value = nextValues
    validationErrors.value = nextErrors
  }

  function exportSettings() {
    return Object.fromEntries(
      Object.entries(values.value).map(([key, value]) => [key, cloneSettingValue(value)]),
    ) as Record<string, SettingValue>
  }

  function watchSetting(key: string, callback: (value: SettingValue | undefined) => void) {
    return watch(() => values.value[key], callback)
  }

  function setActiveCategory(categoryId: SettingCategoryId) {
    activeCategoryId.value = categoryId
    activeGroupId.value = null
    const firstItem = getCategoryById(categories.value, categoryId).groups[0]?.items[0]
    if (firstItem) selectedSettingId.value = firstItem.id
  }

  function setActiveGroup(groupId: string | null) {
    activeGroupId.value = groupId
    if (!groupId) return

    const context = allContexts.value.find(({ group }) => group.id === groupId)
    if (context) {
      activeCategoryId.value = context.category.id
      selectedSettingId.value = context.item.id
    }
  }

  function setSelectedSetting(settingId: string | null) {
    selectedSettingId.value = settingId
    const context = findSettingContext(categories.value, settingId)
    if (context) {
      activeCategoryId.value = context.category.id
      activeGroupId.value = context.group.id
    }
  }

  function setSearchQuery(query: string) {
    searchQuery.value = query
  }

  function setCategoryFilter(categoryId: SettingCategoryId | 'all') {
    categoryFilter.value = categoryId
  }

  function runAction(actionId: string) {
    actionStatuses.value = { ...actionStatuses.value, [actionId]: 'idle' }
    validationErrors.value = {
      ...validationErrors.value,
      [actionId]: '该功能暂未实现',
    }
  }

  return {
    categories,
    defaults,
    values,
    validationErrors,
    activeCategoryId,
    activeGroupId,
    selectedSettingId,
    searchQuery,
    categoryFilter,
    loading,
    actionStatuses,
    allContexts,
    allItems,
    currentCategory,
    selectedContext,
    selectedSetting,
    dirtyKeys,
    modifiedCount,
    hasModifiedSettings,
    read,
    getValue,
    setValue,
    resetSetting,
    resetCategory,
    resetAll,
    importSettings,
    exportSettings,
    watchSetting,
    setActiveCategory,
    setActiveGroup,
    setSelectedSetting,
    setSearchQuery,
    setCategoryFilter,
    runAction,
  }
})
