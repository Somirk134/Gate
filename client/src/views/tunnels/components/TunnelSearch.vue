<!--
  TunnelSearch — 隧道搜索框
  ------------------------------------------------------------------
  基于 GSearchInput，封装 placeholder 与事件透传。
-->
<template>
  <GSearchInput
    :model-value="modelValue"
    :placeholder="placeholderText"
    size="sm"
    @update:model-value="$emit('update:modelValue', $event)" />
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import GSearchInput from '@components/form/GSearchInput.vue'

const props = defineProps<{
  modelValue: string
  placeholder?: string
}>()

defineEmits<{ 'update:modelValue': [value: string] }>()

const { t } = useI18n()
const placeholderText = computed(() => props.placeholder ?? t('tunnel.searchPlaceholder'))
</script>
