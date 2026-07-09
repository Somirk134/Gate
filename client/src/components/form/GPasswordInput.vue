<!--
  GPasswordInput — 密码输入框（带显隐切换）
  ------------------------------------------------------------------
  用途：登录/注册/Token 等敏感输入。复用 GInput 视觉，附加眼睛切换。
-->
<template>
  <GInput
    :model-value="modelValue"
    :type="visible ? 'text' : 'password'"
    :size="size"
    :placeholder="placeholder"
    :disabled="disabled"
    :state="state"
    :autocomplete="autocomplete"
    :prefix="lockIcon ? 'lock' : undefined"
    @update:model-value="emit('update:modelValue', $event)">
    <template #suffix>
      <button
        type="button"
        class="g-password__toggle"
        :title="visible ? '隐藏' : '显示'"
        @click="visible = !visible">
        <GIcon :name="visible ? 'eye-off' : 'eye'" :size="iconSize" />
      </button>
    </template>
  </GInput>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import GInput from './GInput.vue'
import GIcon from '@components/icons/GIcon.vue'

const props = withDefaults(
  defineProps<{
    modelValue?: string
    size?: 'sm' | 'md' | 'lg'
    placeholder?: string
    disabled?: boolean
    state?: 'normal' | 'error' | 'success'
    autocomplete?: string
    lockIcon?: boolean
  }>(),
  {
    size: 'md',
    disabled: false,
    state: 'normal',
    lockIcon: true,
  },
)

const emit = defineEmits<{ 'update:modelValue': [value: string] }>()
const visible = ref(false)
const iconSize = computed(() => (props.size === 'sm' ? 14 : props.size === 'lg' ? 18 : 16))
</script>

<style scoped>
.g-password__toggle {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: transparent;
  color: var(--text-tertiary);
  cursor: pointer;
  padding: 2px;
  border-radius: var(--radius-sm);
  margin-right: var(--space-2);
  transition:
    color var(--duration-fast) var(--ease-out),
    background-color var(--duration-fast) var(--ease-out);
}
.g-password__toggle:hover {
  color: var(--text-primary);
  background: var(--bg-surface-hover);
}
</style>
