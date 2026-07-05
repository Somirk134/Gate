<!--
  GIconButton — 纯图标按钮
  ------------------------------------------------------------------
  用途：工具栏、行内操作等仅含图标的方形按钮。统一尺寸与 hover。
  基于设计令牌，与 GButton 视觉一致。

  Props:
    variant  ghost(默认透明) | soft(柔和底) | solid(实心) | outline(描边)
    size     sm(28) | md(32) | lg(36)
    icon     图标名（见 registry）
    tooltip  悬停提示（title 属性）
-->
<template>
  <button
    class="g-icon-btn"
    :class="[`g-icon-btn--${variant}`, `g-icon-btn--${size}`, { 'g-icon-btn--active': active }]"
    :disabled="disabled"
    :title="tooltip"
    type="button"
    @click="handleClick"
  >
    <GIcon :name="name" :size="iconSize" :disabled="disabled" />
  </button>
</template>

<script setup lang="ts">
import { computed } from "vue"
import GIcon from "@components/icons/GIcon.vue"

const props = withDefaults(
  defineProps<{
    name: string
    variant?: "ghost" | "soft" | "solid" | "outline"
    size?: "sm" | "md" | "lg"
    disabled?: boolean
    active?: boolean
    tooltip?: string
  }>(),
  {
    variant: "ghost",
    size: "md",
    disabled: false,
    active: false,
  },
)

const emit = defineEmits<{ click: [event: MouseEvent] }>()

const iconSize = computed(() =>
  props.size === "sm" ? 14 : props.size === "lg" ? 18 : 16,
)

function handleClick(e: MouseEvent) {
  if (props.disabled) return
  emit("click", e)
}
</script>

<style scoped>
.g-icon-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border: 1px solid transparent;
  border-radius: var(--radius-md);
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
  flex-shrink: 0;
  transition: background-color var(--duration-fast) var(--ease-out),
    color var(--duration-fast) var(--ease-out),
    border-color var(--duration-fast) var(--ease-out),
    transform var(--duration-fast) var(--ease-out);
}

.g-icon-btn--sm { width: var(--control-height-sm); height: var(--control-height-sm); }
.g-icon-btn--md { width: var(--control-height-md); height: var(--control-height-md); }
.g-icon-btn--lg { width: var(--control-height-lg); height: var(--control-height-lg); }

.g-icon-btn:active:not(:disabled) { transform: scale(0.92); }
.g-icon-btn:disabled { opacity: 0.4; cursor: not-allowed; }

/* ── ghost ── */
.g-icon-btn--ghost:hover:not(:disabled) {
  background: var(--bg-surface-hover);
  color: var(--text-primary);
}

/* ── soft ── */
.g-icon-btn--soft {
  background: var(--bg-surface);
}
.g-icon-btn--soft:hover:not(:disabled) {
  background: var(--bg-surface-hover);
  color: var(--text-primary);
}

/* ── outline ── */
.g-icon-btn--outline {
  border-color: var(--color-border);
}
.g-icon-btn--outline:hover:not(:disabled) {
  background: var(--bg-surface-hover);
  border-color: var(--color-border-strong);
  color: var(--text-primary);
}

/* ── solid ── */
.g-icon-btn--solid {
  background: var(--color-primary);
  color: var(--color-text-on-primary);
}
.g-icon-btn--solid:hover:not(:disabled) {
  background: var(--color-primary-hover);
}

/* ── active ── */
.g-icon-btn--active {
  background: var(--color-primary-muted);
  color: var(--color-primary);
}
</style>
