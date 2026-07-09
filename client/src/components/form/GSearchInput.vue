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
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import GInput from './GInput.vue'

const props = withDefaults(
  defineProps<{
    modelValue?: string
    size?: 'sm' | 'md' | 'lg'
    placeholder?: string
    clearable?: boolean
  }>(),
  {
    size: 'md',
    clearable: true,
  },
)

const emit = defineEmits<{
  'update:modelValue': [value: string]
  search: [event: KeyboardEvent]
}>()
const { t } = useI18n()
const placeholder = computed(() => props.placeholder ?? t('form.searchPlaceholder'))
</script>
