<template>
  <select class="setting-control setting-select" :value="String(modelValue)" :disabled="disabled" @change="handleChange">
    <option
      v-for="option in options"
      :key="String(option.value)"
      :value="String(option.value)"
      :disabled="option.disabled"
    >
      {{ option.label }}{{ option.badge ? ` (${option.badge})` : "" }}
    </option>
  </select>
</template>

<script setup lang="ts">
import type { SettingOption, SettingPrimitive } from "../types"

const props = withDefaults(
  defineProps<{
    modelValue: SettingPrimitive
    options: SettingOption[]
    disabled?: boolean
  }>(),
  {
    disabled: false,
  },
)

const emit = defineEmits<{
  "update:modelValue": [value: SettingPrimitive]
}>()

function handleChange(event: Event) {
  const nextValue = (event.target as HTMLSelectElement).value
  const option = props.options.find((candidate) => String(candidate.value) === nextValue)
  if (option) emit("update:modelValue", option.value)
}
</script>
