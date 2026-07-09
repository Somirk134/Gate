<template>
  <div class="settings-search">
    <div class="settings-search__input">
      <GIcon name="search" :size="15" />
      <input
        ref="inputRef"
        :value="query"
        type="search"
        placeholder="搜索设置"
        @input="emit('update:query', ($event.target as HTMLInputElement).value)" />
      <button v-if="query" type="button" @click="emit('update:query', '')">
        <GIcon name="close" :size="14" />
      </button>
    </div>

    <select
      class="settings-search__filter"
      :value="categoryFilter"
      @change="
        emit(
          'update:categoryFilter',
          ($event.target as HTMLSelectElement).value as SettingCategoryId | 'all',
        )
      ">
      <option value="all">全部分类</option>
      <option v-for="category in categories" :key="category.id" :value="category.id">
        {{ category.label }}
      </option>
    </select>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import GIcon from '@components/icons/GIcon.vue'
import type { SettingCategory, SettingCategoryId } from '../types'

defineProps<{
  query: string
  categoryFilter: SettingCategoryId | 'all'
  categories: SettingCategory[]
}>()

const emit = defineEmits<{
  'update:query': [value: string]
  'update:categoryFilter': [value: SettingCategoryId | 'all']
}>()

const inputRef = ref<HTMLInputElement | null>(null)

function focus() {
  inputRef.value?.focus()
  inputRef.value?.select()
}

defineExpose({ focus })
</script>
