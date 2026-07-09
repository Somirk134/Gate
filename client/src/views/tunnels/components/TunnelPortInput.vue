<!--
  TunnelPortInput — 隧道端口输入
  ------------------------------------------------------------------
  复用 GPortInput，限定 1-65535，等宽字体，前置 plug 图标。
  支持自定义 placeholder 与 label。
-->
<template>
  <GPortInput :model-value="modelValue" :size="size" :disabled="disabled" />
</template>

<script setup lang="ts">
import GPortInput from '@components/form/GPortInput.vue'

withDefaults(
  defineProps<{
    modelValue?: number | null
    size?: 'sm' | 'md' | 'lg'
    disabled?: boolean
  }>(),
  { size: 'md', disabled: false },
)

defineEmits<{ 'update:modelValue': [value: number | null] }>()
</script>
