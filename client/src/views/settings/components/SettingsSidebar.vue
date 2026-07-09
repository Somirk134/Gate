<template>
  <aside class="settings-sidebar">
    <div class="settings-sidebar__header">
      <span>{{ t('settings.legacy.navigation') }}</span>
      <strong>{{ categories.length }}</strong>
    </div>

    <nav class="settings-tree">
      <div v-for="category in categories" :key="category.id" class="settings-tree__section">
        <div class="settings-tree__category">
          <button class="settings-tree__toggle" type="button" @click="toggle(category.id)">
            <GIcon name="chevron-right" :size="14" :class="{ expanded: expanded[category.id] }" />
          </button>
          <button
            class="settings-tree__node"
            type="button"
            :class="{ active: activeCategoryId === category.id && !activeGroupId }"
            @click="emit('select-category', category.id)">
            <GIcon :name="category.icon" :size="15" />
            <span>{{ category.label }}</span>
            <strong>{{ countCategoryItems(category) }}</strong>
          </button>
        </div>

        <transition name="settings-collapse">
          <div v-if="expanded[category.id]" class="settings-tree__children">
            <button
              v-for="group in category.groups"
              :key="group.id"
              class="settings-tree__node settings-tree__node--group"
              type="button"
              :class="{ active: activeGroupId === group.id }"
              @click="emit('select-group', group.id)">
              <span>{{ group.label }}</span>
              <strong>{{ group.items.length }}</strong>
            </button>
          </div>
        </transition>
      </div>
    </nav>
  </aside>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import GIcon from '@components/icons/GIcon.vue'
import type { SettingCategory, SettingCategoryId } from '../types'

const props = defineProps<{
  categories: SettingCategory[]
  activeCategoryId: SettingCategoryId
  activeGroupId: string | null
}>()

const emit = defineEmits<{
  'select-category': [categoryId: SettingCategoryId]
  'select-group': [groupId: string]
}>()

const { t } = useI18n()
const expanded = ref<Record<string, boolean>>({})

watch(
  () => props.categories,
  (categories) => {
    expanded.value = Object.fromEntries(
      categories.map((category) => [category.id, category.id === props.activeCategoryId]),
    )
  },
  { immediate: true },
)

watch(
  () => props.activeCategoryId,
  (categoryId) => {
    expanded.value = { ...expanded.value, [categoryId]: true }
  },
)

function toggle(categoryId: SettingCategoryId) {
  expanded.value = {
    ...expanded.value,
    [categoryId]: !expanded.value[categoryId],
  }
}

function countCategoryItems(category: SettingCategory) {
  return category.groups.reduce((total, group) => total + group.items.length, 0)
}
</script>
