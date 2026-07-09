<!--
  GButton — 统一按钮组件
  ------------------------------------------------------------------
  用途：项目内所有按钮必须使用 GButton，禁止页面自写 .btn 样式。

  Variants:
    primary   主操作（强调色实心）
    secondary 次操作（描边）
    ghost     幽灵（无边框，hover 出底）
    danger    危险操作（红色实心）
    text      纯文字按钮
  Sizes: sm(28) / md(32) / lg(36)
  特性：loading / disabled / icon / block / 可渲染为链接(native)

  用法：
    <GButton variant="primary" @click="...">保存</GButton>
    <GButton variant="ghost" size="sm" icon="plus">新建</GButton>
    <GButton variant="danger" :loading="saving">删除</GButton>
-->
<template>
  <button
    class="g-btn"
    :class="[
      `g-btn--${variant}`,
      `g-btn--${size}`,
      { 'g-btn--block': block, 'g-btn--icon-only': iconOnly, 'g-btn--loading': loading },
    ]"
    :disabled="disabled || loading"
    :type="type"
    @click="handleClick">
    <!-- Loading spinner -->
    <span v-if="loading" class="g-btn__spinner">
      <GIcon name="loader" :size="iconSize" spin />
    </span>

    <!-- Leading icon -->
    <GIcon v-if="icon && !loading" :name="icon" :size="iconSize" class="g-btn__icon" />

    <span v-if="!iconOnly" class="g-btn__label">
      <slot />
    </span>

    <!-- Trailing icon -->
    <GIcon
      v-if="trailingIcon"
      :name="trailingIcon"
      :size="iconSize"
      class="g-btn__icon g-btn__icon--trailing" />
  </button>
</template>

<script setup lang="ts">
import { computed, useSlots } from 'vue'
import GIcon from '@components/icons/GIcon.vue'

const props = withDefaults(
  defineProps<{
    variant?: 'primary' | 'secondary' | 'ghost' | 'danger' | 'text'
    size?: 'sm' | 'md' | 'lg'
    icon?: string
    trailingIcon?: string
    loading?: boolean
    disabled?: boolean
    block?: boolean
    type?: 'button' | 'submit' | 'reset'
  }>(),
  {
    variant: 'secondary',
    size: 'md',
    loading: false,
    disabled: false,
    block: false,
    type: 'button',
  },
)

const emit = defineEmits<{ click: [event: MouseEvent] }>()
const slots = useSlots()

const iconOnly = computed(() => !!props.icon && !slots.default)
const iconSize = computed(() => (props.size === 'sm' ? 14 : props.size === 'lg' ? 18 : 16))

function handleClick(e: MouseEvent) {
  if (props.disabled || props.loading) return
  emit('click', e)
}
</script>

<style scoped>
.g-btn {
  --_height: var(--control-height-md);
  --_padding: var(--space-4);

  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-2);
  height: var(--_height);
  padding: 0 var(--_padding);
  border: 1px solid transparent;
  border-radius: var(--radius-button);
  background: transparent;
  color: var(--text-primary);
  font-family: var(--font-ui);
  font-size: var(--font-size-button);
  font-weight: var(--weight-medium);
  line-height: 1;
  white-space: nowrap;
  cursor: pointer;
  user-select: none;
  transition:
    background-color var(--duration-fast) var(--ease-out),
    border-color var(--duration-fast) var(--ease-out),
    color var(--duration-fast) var(--ease-out),
    transform var(--duration-fast) var(--ease-out),
    box-shadow var(--duration-fast) var(--ease-out);
}

.g-btn:active:not(:disabled) {
  transform: scale(0.97);
}
.g-btn:disabled {
  opacity: 0.45;
  cursor: not-allowed;
}

/* ── Sizes ── */
.g-btn--sm {
  --_height: var(--control-height-sm);
  --_padding: var(--space-3);
  font-size: var(--text-sm);
}
.g-btn--lg {
  --_height: var(--control-height-lg);
  --_padding: var(--space-5);
  font-size: var(--text-md);
}
.g-btn--icon-only {
  padding: 0;
  width: var(--_height);
  aspect-ratio: 1;
}

.g-btn--block {
  display: flex;
  width: 100%;
}

/* ── Variants ── */
.g-btn--primary {
  background: var(--color-primary);
  border-color: var(--color-primary);
  color: var(--color-text-on-primary);
}
.g-btn--primary:hover:not(:disabled) {
  background: var(--color-primary-hover);
  border-color: var(--color-primary-hover);
}
.g-btn--primary:active:not(:disabled) {
  background: var(--color-primary-active);
  border-color: var(--color-primary-active);
}

.g-btn--secondary {
  background: var(--bg-surface);
  border-color: var(--color-border);
  color: var(--text-primary);
}
.g-btn--secondary:hover:not(:disabled) {
  background: var(--bg-surface-hover);
  border-color: var(--color-border-strong);
}

.g-btn--ghost {
  background: transparent;
  border-color: transparent;
  color: var(--text-secondary);
}
.g-btn--ghost:hover:not(:disabled) {
  background: var(--bg-surface-hover);
  color: var(--text-primary);
}

.g-btn--danger {
  background: var(--color-error);
  border-color: var(--color-error);
  color: var(--color-error-fg);
}
.g-btn--danger:hover:not(:disabled) {
  background: var(--color-error-hover);
  border-color: var(--color-error-hover);
}

.g-btn--text {
  background: transparent;
  border-color: transparent;
  color: var(--color-primary);
  padding: 0 var(--space-2);
  height: auto;
}
.g-btn--text:hover:not(:disabled) {
  background: var(--color-primary-muted);
}

/* ── Loading ── */
.g-btn--loading {
  pointer-events: none;
}
.g-btn__spinner {
  display: inline-flex;
  align-items: center;
}
</style>
