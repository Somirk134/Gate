<template>
  <input
    class="setting-control setting-input"
    :type="inputType"
    :value="modelValue"
    :placeholder="placeholder"
    :disabled="disabled"
    @input="emit('update:modelValue', ($event.target as HTMLInputElement).value)" />
</template>

<script setup lang="ts">
withDefaults(
  defineProps<{
    modelValue: string
    placeholder?: string
    inputType?: 'text' | 'password' | 'url'
    disabled?: boolean
  }>(),
  {
    placeholder: '',
    inputType: 'text',
    disabled: false,
  },
)

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()
</script>
