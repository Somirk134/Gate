<!--
  GPortInput — 端口输入框
  ------------------------------------------------------------------
  用途：隧道/服务端口输入。限定 1-65535，等宽字体，前置 plug 图标。
-->
<template>
  <GInput
    :model-value="modelValue != null ? String(modelValue) : ''"
    type="number"
    :size="size"
    placeholder="0"
    :state="state"
    :disabled="disabled"
    prefix="plug"
    @update:model-value="onInput"
  />
</template>

<script setup lang="ts">
import { computed } from "vue"
import GInput from "./GInput.vue"

const props = withDefaults(
  defineProps<{
    modelValue?: number | null
    size?: "sm" | "md" | "lg"
    disabled?: boolean
  }>(),
  {
    size: "md",
    disabled: false,
  },
)

const emit = defineEmits<{ "update:modelValue": [value: number | null] }>()

const state = computed<"normal" | "error" | "success">(() => {
  if (props.modelValue == null || props.modelValue === 0) return "normal"
  if (props.modelValue < 1 || props.modelValue > 65535) return "error"
  return "success"
})

function onInput(val: string) {
  if (val === "") return emit("update:modelValue", null)
  const n = parseInt(val, 10)
  emit("update:modelValue", Number.isNaN(n) ? null : n)
}
</script>

<style scoped>
:deep(.g-input__field) {
  font-family: var(--font-mono);
}
</style>
