<!--
  GNumberInput — 数字输入框（带步进器）
  ------------------------------------------------------------------
  用途：端口号、超时、并发数等数字输入。统一聚焦与步进按钮。
-->
<template>
  <GInput
    :model-value="String(modelValue ?? '')"
    type="number"
    :size="size"
    :placeholder="placeholder"
    :disabled="disabled"
    :state="state"
    @update:model-value="onInput">
    <template #suffix>
      <div class="g-number__stepper">
        <button type="button" class="g-number__btn" :disabled="disabled || atMax" @click="step(1)">
          <GIcon name="chevron-up" :size="12" />
        </button>
        <button type="button" class="g-number__btn" :disabled="disabled || atMin" @click="step(-1)">
          <GIcon name="chevron-down" :size="12" />
        </button>
      </div>
    </template>
  </GInput>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import GInput from './GInput.vue'
import GIcon from '@components/icons/GIcon.vue'

const props = withDefaults(
  defineProps<{
    modelValue?: number | null
    size?: 'sm' | 'md' | 'lg'
    placeholder?: string
    disabled?: boolean
    state?: 'normal' | 'error' | 'success'
    min?: number
    max?: number
    step?: number
  }>(),
  {
    size: 'md',
    disabled: false,
    state: 'normal',
    min: -Infinity,
    max: Infinity,
    step: 1,
  },
)

const emit = defineEmits<{ 'update:modelValue': [value: number | null] }>()

const atMin = computed(() => props.modelValue != null && props.modelValue <= props.min)
const atMax = computed(() => props.modelValue != null && props.modelValue >= props.max)

function onInput(val: string) {
  if (val === '') return emit('update:modelValue', null)
  const n = Number(val)
  emit('update:modelValue', Number.isNaN(n) ? null : n)
}

function step(dir: 1 | -1) {
  if (props.disabled) return
  const base = props.modelValue ?? 0
  let next = base + dir * props.step
  if (next < props.min) next = props.min
  if (next > props.max) next = props.max
  emit('update:modelValue', next)
}
</script>

<style scoped>
.g-number__stepper {
  display: flex;
  flex-direction: column;
  margin-right: var(--space-1);
}
.g-number__btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 12px;
  border: none;
  background: transparent;
  color: var(--text-tertiary);
  cursor: pointer;
  transition:
    color var(--duration-fast) var(--ease-out),
    background-color var(--duration-fast) var(--ease-out);
}
.g-number__btn:hover:not(:disabled) {
  color: var(--text-primary);
  background: var(--bg-surface-hover);
}
.g-number__btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}
</style>
