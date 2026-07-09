<!--
  GFormField — 表单字段容器
  ------------------------------------------------------------------
  用途：统一 label / 控件 / 提示文本/错误文本 的纵向布局与间距。
  所有表单控件应包裹在 GFormField 中。

  Slots:
    label    标签内容
    default  控件
    hint     辅助提示
    error    错误提示（出现时自动替换 hint 并标红）
-->
<template>
  <div class="g-form-field" :class="{ 'g-form-field--error': !!error }">
    <div v-if="$slots.label" class="g-form-field__label">
      <GLabel :required="required">
        <slot name="label" />
      </GLabel>
    </div>

    <div class="g-form-field__control">
      <slot />
    </div>

    <div v-if="error" class="g-form-field__message g-form-field__message--error">
      <GIcon name="alert-circle" :size="12" />
      <span>{{ error }}</span>
    </div>
    <div v-else-if="$slots.hint" class="g-form-field__message">
      <slot name="hint" />
    </div>
  </div>
</template>

<script setup lang="ts">
import GIcon from '@components/icons/GIcon.vue'
import GLabel from './GLabel.vue'

defineProps<{
  required?: boolean
  error?: string
}>()
</script>

<style scoped>
.g-form-field {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}
.g-form-field__label {
  display: flex;
  align-items: center;
}
.g-form-field__control {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}
.g-form-field__message {
  display: flex;
  align-items: center;
  gap: var(--space-1);
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  line-height: var(--leading-tight);
}
.g-form-field__message--error {
  color: var(--color-error);
}
</style>
