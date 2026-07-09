<!--
  GTextarea — 多行文本输入
  ------------------------------------------------------------------
  用途：描述、配置、日志备注等多行输入。统一聚焦/尺寸/状态。
-->
<template>
  <div
    class="g-textarea"
    :class="[
      `g-textarea--${state}`,
      {
        'g-textarea--disabled': disabled,
        'g-textarea--focused': focused,
        'g-textarea--resizable': resizable,
      },
    ]">
    <textarea
      class="g-textarea__field"
      :value="modelValue"
      :placeholder="placeholder"
      :disabled="disabled"
      :readonly="readonly"
      :rows="rows"
      :maxlength="maxlength"
      :spellcheck="spellcheck"
      @input="onInput"
      @focus="focused = true"
      @blur="focused = false" />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'

withDefaults(
  defineProps<{
    modelValue?: string
    placeholder?: string
    disabled?: boolean
    readonly?: boolean
    state?: 'normal' | 'error' | 'success'
    rows?: number
    resizable?: boolean
    maxlength?: number
    spellcheck?: boolean
  }>(),
  {
    disabled: false,
    readonly: false,
    state: 'normal',
    rows: 4,
    resizable: false,
    spellcheck: false,
  },
)

const emit = defineEmits<{ 'update:modelValue': [value: string] }>()
const focused = ref(false)

function onInput(e: Event) {
  emit('update:modelValue', (e.target as HTMLTextAreaElement).value)
}
</script>

<style scoped>
.g-textarea {
  display: block;
  width: 100%;
  background: var(--bg-input);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-input);
  transition:
    border-color var(--duration-fast) var(--ease-out),
    box-shadow var(--duration-fast) var(--ease-out);
}
.g-textarea--focused {
  border-color: var(--color-border-focus);
  box-shadow: var(--shadow-focus);
}
.g-textarea--error {
  border-color: var(--color-error);
}
.g-textarea--success {
  border-color: var(--color-success);
}
.g-textarea--disabled {
  opacity: 0.55;
  cursor: not-allowed;
  background: var(--bg-surface);
}

.g-textarea__field {
  display: block;
  width: 100%;
  background: transparent;
  border: none;
  outline: none;
  resize: none;
  color: var(--text-primary);
  font-family: var(--font-ui);
  font-size: var(--font-size-input);
  line-height: var(--leading-normal);
  padding: var(--space-3);
}
.g-textarea__field::placeholder {
  color: var(--text-tertiary);
}
.g-textarea--resizable .g-textarea__field {
  resize: vertical;
}
.g-textarea--disabled .g-textarea__field {
  cursor: not-allowed;
}
</style>
