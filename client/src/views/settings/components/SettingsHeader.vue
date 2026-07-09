<template>
  <header class="settings-header">
    <div class="settings-header__title">
      <h1>设置</h1>
      <span>{{ resultCount }} 项设置</span>
    </div>

    <SettingsSearch
      ref="searchRef"
      :query="query"
      :category-filter="categoryFilter"
      :categories="categories"
      @update:query="emit('update:query', $event)"
      @update:category-filter="emit('update:categoryFilter', $event)" />

    <div class="settings-header__actions">
      <span v-if="modifiedCount" class="settings-header__modified"
        >{{ modifiedCount }} 项已修改</span
      >
      <GButton
        size="sm"
        variant="ghost"
        icon="refresh"
        :disabled="!modifiedCount"
        @click="emit('reset-all')">
        重置
      </GButton>
    </div>
  </header>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import GButton from '@components/base/GButton.vue'
import type { SettingCategory, SettingCategoryId } from '../types'
import SettingsSearch from './SettingsSearch.vue'

defineProps<{
  categories: SettingCategory[]
  query: string
  categoryFilter: SettingCategoryId | 'all'
  resultCount: number
  modifiedCount: number
}>()

const emit = defineEmits<{
  'update:query': [value: string]
  'update:categoryFilter': [value: SettingCategoryId | 'all']
  'reset-all': []
}>()

const searchRef = ref<InstanceType<typeof SettingsSearch> | null>(null)

function focusSearch() {
  searchRef.value?.focus()
}

defineExpose({ focusSearch })
</script>
