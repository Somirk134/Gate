<!--
  GSectionHeader — 区块标题
  ------------------------------------------------------------------
  用途：卡片/区块内的小标题 + 右侧操作。统一 section 标题样式。
-->
<template>
  <div class="g-section-header">
    <div class="g-section-header__title">
      <GIcon v-if="icon" :name="icon" :size="16" class="g-section-header__icon" />
      <slot />
    </div>
    <div v-if="$slots.actions" class="g-section-header__actions">
      <slot name="actions" />
    </div>
  </div>
</template>

<script setup lang="ts">
import GIcon from "@components/icons/GIcon.vue"

defineProps<{ icon?: string }>()
</script>

<style scoped>
.g-section-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
}
.g-section-header__title {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  font-size: var(--text-md);
  font-weight: var(--weight-semibold);
  color: var(--text-primary);
}
.g-section-header__icon {
  color: var(--text-tertiary);
}
.g-section-header__actions {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}
</style>
