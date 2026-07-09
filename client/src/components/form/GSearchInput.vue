<!--
  GSearchInput — 搜索输入框
  ------------------------------------------------------------------
  用途：列表/表格顶部搜索。自带搜索图标与清除，回车触发 search。
-->
<template>
  <GInput
    :model-value="modelValue"
    :size="size"
    :placeholder="placeholder"
    :clearable="clearable"
    prefix="search"
    @update:model-value="emit('update:modelValue', $event)"
    @enter="emit('search', $event)" />
</template>

<script setup lang="ts">
import GInput from './GInput.vue'

withDefaults(
  defineProps<{
    modelValue?: string
    size?: 'sm' | 'md' | 'lg'
    placeholder?: string
    clearable?: boolean
  }>(),
  {
    size: 'md',
    placeholder: '搜索…',
    clearable: true,
  },
)

const emit = defineEmits<{
  'update:modelValue': [value: string]
  search: [event: KeyboardEvent]
}>()
</script>
