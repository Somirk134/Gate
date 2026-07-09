<!--
  ProjectSort — 排序选择器
  ------------------------------------------------------------------
   支持：名称 / 创建时间 / 更新时间 / 运行状态 / 隧道数量
  下拉形态，点击切换方向。
-->
<template>
  <div class="project-sort">
    <GIconButton
      name="arrow-up-down"
      size="sm"
      :tooltip="t('project.sort.directionTooltip')"
      @click="toggleDirection" />
    <div class="project-sort__select-wrap">
      <select :value="modelValue" class="project-sort__select" @change="onChange">
        <option v-for="item in items" :key="item.key" :value="item.key">
          {{ item.label }}
        </option>
      </select>
      <GIcon name="chevron-down" :size="12" class="project-sort__chevron" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import GIcon from '@components/icons/GIcon.vue'
import GIconButton from '@components/base/GIconButton.vue'
import type { ProjectSortType, SortDirection } from '../types'

const props = defineProps<{
  modelValue: ProjectSortType
  direction: SortDirection
}>()

const emit = defineEmits<{
  'update:modelValue': [value: ProjectSortType]
  'update:direction': [value: SortDirection]
}>()

const { t } = useI18n()

const items = computed<Array<{ key: ProjectSortType; label: string }>>(() => [
  { key: 'name', label: t('project.sort.name') },
  { key: 'createdAt', label: t('project.sort.createdAt') },
  { key: 'updatedAt', label: t('project.sort.updatedAt') },
  { key: 'status', label: t('project.sort.status') },
  { key: 'tunnelCount', label: t('project.sort.tunnelCount') },
])

function onChange(e: Event) {
  emit('update:modelValue', (e.target as HTMLSelectElement).value as ProjectSortType)
}

function toggleDirection() {
  emit('update:direction', props.direction === 'asc' ? 'desc' : 'asc')
}
</script>

<style scoped>
.project-sort {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
}

.project-sort__select-wrap {
  position: relative;
  display: inline-flex;
  align-items: center;
}

.project-sort__select {
  appearance: none;
  height: var(--control-height-sm);
  padding: 0 var(--space-6) 0 var(--space-3);
  background: var(--bg-input);
  border: 1px solid var(--color-border-subtle);
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  font-family: var(--font-ui);
  font-size: var(--text-sm);
  cursor: pointer;
  transition: border-color var(--duration-fast) var(--ease-out);
}

.project-sort__select:hover {
  border-color: var(--color-border-strong);
  color: var(--text-primary);
}

.project-sort__chevron {
  position: absolute;
  right: var(--space-2);
  color: var(--text-tertiary);
  pointer-events: none;
}
</style>
