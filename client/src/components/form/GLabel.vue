<!--
  GLabel — 表单标签
  ------------------------------------------------------------------
  用途：统一表单标签样式与必填标记。
-->
<template>
  <label
    class="g-label"
    :class="{ 'g-label--required': required, 'g-label--disabled': disabled }"
  >
    <slot />
    <span
      v-if="required"
      class="g-label__required"
      aria-hidden="true"
    >*</span>
  </label>
</template>

<script setup lang="ts">
withDefaults(defineProps<{ required?: boolean; disabled?: boolean }>(), {
  required: false,
  disabled: false,
})
</script>

<style scoped>
.g-label {
  display: inline-flex;
  align-items: center;
  gap: 2px;
  font-family: var(--font-ui);
  font-size: var(--text-sm);
  font-weight: var(--weight-medium);
  color: var(--text-secondary);
  line-height: var(--leading-tight);
}
.g-label--disabled {
  color: var(--text-disabled);
  cursor: not-allowed;
}
.g-label__required {
  color: var(--color-error);
  margin-left: 2px;
}
</style>
