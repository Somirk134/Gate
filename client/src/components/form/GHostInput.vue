<!--
  GHostInput — 主机/域名输入框
  ------------------------------------------------------------------
  用途：服务器地址、域名、IP 输入。前置 globe 图标，等宽字体。
-->
<template>
  <GInput
    :model-value="modelValue"
    :size="size"
    :placeholder="placeholderText"
    :state="state"
    :disabled="disabled"
    :clearable="clearable"
    prefix="globe"
    @update:model-value="emit('update:modelValue', $event)" />
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
    disabled?: boolean
    state?: 'normal' | 'error' | 'success'
    clearable?: boolean
  }>(),
  {
    size: 'md',
    disabled: false,
    state: 'normal',
    clearable: true,
  },
)

const emit = defineEmits<{ 'update:modelValue': [value: string] }>()
const { t } = useI18n()
const placeholderText = computed(() => props.placeholder ?? t('form.hostPlaceholder'))
</script>

<style scoped>
:deep(.g-input__field) {
  font-family: var(--font-mono);
}
</style>
