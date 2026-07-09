<!--
  ProjectToolbar — 列表页工具栏
  ------------------------------------------------------------------
  整合 搜索 + 筛选 + 排序 + 创建按钮。
-->
<template>
  <div class="projects-toolbar">
    <ProjectSearch
      :model-value="query"
      class="projects-toolbar__search"
      @update:model-value="$emit('update:query', $event)" />

    <div class="projects-toolbar__right">
      <ProjectFilter
        :model-value="filter"
        :counts="counts"
        @update:model-value="$emit('update:filter', $event)" />

      <ProjectSort
        :model-value="sortBy"
        :direction="direction"
        @update:model-value="$emit('update:sortBy', $event)"
        @update:direction="$emit('update:direction', $event)" />

      <GButton variant="primary" icon="plus" @click="$emit('create')">
        {{ t('project.newProject') }}
      </GButton>
    </div>
  </div>
</template>

<script setup lang="ts">
import GButton from '@components/base/GButton.vue'
import { useI18n } from 'vue-i18n'
import ProjectSearch from './ProjectSearch.vue'
import ProjectFilter from './ProjectFilter.vue'
import ProjectSort from './ProjectSort.vue'
import type { ProjectFilterType, ProjectSortType, SortDirection } from '../types'

const { t } = useI18n()

defineProps<{
  query: string
  filter: ProjectFilterType
  sortBy: ProjectSortType
  direction: SortDirection
  counts: Record<ProjectFilterType, number>
}>()

defineEmits<{
  'update:query': [value: string]
  'update:filter': [value: ProjectFilterType]
  'update:sortBy': [value: ProjectSortType]
  'update:direction': [value: SortDirection]
  create: []
}>()
</script>
