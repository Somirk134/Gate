<!--
  QuickActionCard — 快捷操作卡片
  ------------------------------------------------------------------
  大图标 + 文字 + 快捷键提示，带 hover 动画。
-->
<template>
  <GCard
    variant="interactive"
    padding="md"
    clickable
    class="quick-action"
    @click="$emit('click')"
  >
    <div class="quick-action__inner">
      <span
        class="quick-action__icon"
        :class="`quick-action__icon--${variant}`"
      >
        <GIcon
          :name="icon"
          :size="22"
        />
      </span>
      <span class="quick-action__label">{{ label }}</span>
      <span
        v-if="shortcut"
        class="quick-action__shortcut"
      >{{ shortcut }}</span>
    </div>
  </GCard>
</template>

<script setup lang="ts">
import GCard from "@components/base/GCard.vue"
import GIcon from "@components/icons/GIcon.vue"

withDefaults(
  defineProps<{
    icon: string
    label: string
    shortcut?: string
    variant?: "primary" | "success" | "warning" | "info"
  }>(),
  {
    variant: "primary",
  },
)

defineEmits<{ click: [] }>()
</script>

<style scoped>
.quick-action {
  transition: transform var(--duration-base) var(--ease-spring),
    border-color var(--duration-fast) var(--ease-out),
    box-shadow var(--duration-fast) var(--ease-out),
    background-color var(--duration-fast) var(--ease-out);
}
.quick-action:hover {
  transform: translateY(-3px);
  box-shadow: var(--shadow-hover);
}
.quick-action__inner {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-2);
  text-align: center;
}
.quick-action__icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 44px;
  height: 44px;
  border-radius: var(--radius-lg);
  transition: transform var(--duration-base) var(--ease-spring);
}
.quick-action:hover .quick-action__icon {
  transform: scale(1.1);
}
.quick-action__icon--primary {
  background: var(--color-primary-muted);
  color: var(--color-primary);
}
.quick-action__icon--success {
  background: var(--color-success-muted);
  color: var(--color-success);
}
.quick-action__icon--warning {
  background: var(--color-warning-muted);
  color: var(--color-warning);
}
.quick-action__icon--info {
  background: var(--color-info-muted);
  color: var(--color-info);
}
.quick-action__label {
  font-size: var(--text-sm);
  font-weight: var(--weight-medium);
  color: var(--text-primary);
}
.quick-action__shortcut {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  font-family: var(--font-mono);
  padding: 1px var(--space-2);
  background: var(--bg-input);
  border-radius: var(--radius-sm);
  border: 1px solid var(--border-subtle);
}
</style>
