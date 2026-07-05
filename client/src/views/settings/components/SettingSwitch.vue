<template>
  <button
    class="setting-switch"
    type="button"
    role="switch"
    :aria-checked="modelValue"
    :disabled="disabled"
    :class="{ 'setting-switch--on': modelValue }"
    @click="emit('update:modelValue', !modelValue)"
  >
    <span class="setting-switch__thumb" />
  </button>
</template>

<script setup lang="ts">
withDefaults(
  defineProps<{
    modelValue: boolean
    disabled?: boolean
  }>(),
  {
    disabled: false,
  },
)

const emit = defineEmits<{
  "update:modelValue": [value: boolean]
}>()
</script>
