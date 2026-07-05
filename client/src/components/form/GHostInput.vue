<!--
  GHostInput — 主机/域名输入框
  ------------------------------------------------------------------
  用途：服务器地址、域名、IP 输入。前置 globe 图标，等宽字体。
-->
<template>
  <GInput
    :model-value="modelValue"
    :size="size"
    :placeholder="placeholder ?? 'example.com 或 127.0.0.1'"
    :state="state"
    :disabled="disabled"
    :clearable="clearable"
    prefix="globe"
    @update:model-value="emit('update:modelValue', $event)"
  />
</template>

<script setup lang="ts">
import GInput from "./GInput.vue"

withDefaults(
  defineProps<{
    modelValue?: string
    size?: "sm" | "md" | "lg"
    placeholder?: string
    disabled?: boolean
    state?: "normal" | "error" | "success"
    clearable?: boolean
  }>(),
  {
    size: "md",
    disabled: false,
    state: "normal",
    clearable: true,
  },
)

const emit = defineEmits<{ "update:modelValue": [value: string] }>()
</script>

<style scoped>
:deep(.g-input__field) {
  font-family: var(--font-mono);
}
</style>
