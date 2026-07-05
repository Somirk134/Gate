<!--
  GPageHeader — 页面标题区
  ------------------------------------------------------------------
  用途：页面顶部标题 + 描述 + 操作区。统一各页面顶部布局。
  Slots:
    default  标题（可含面包屑）
    description  描述
    actions  右侧操作按钮
-->
<template>
  <div class="g-page-header">
    <div class="g-page-header__main">
      <h1 class="g-page-header__title">
        <slot />
      </h1>
      <p v-if="$slots.description || description" class="g-page-header__desc">
        <slot name="description">{{ description }}</slot>
      </p>
    </div>
    <div v-if="$slots.actions" class="g-page-header__actions">
      <slot name="actions" />
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{ description?: string }>()
</script>

<style scoped>
.g-page-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-4);
  margin-bottom: var(--space-6);
}
.g-page-header__main {
  min-width: 0;
}
.g-page-header__title {
  font-size: var(--text-2xl);
  font-weight: var(--weight-semibold);
  color: var(--text-primary);
  letter-spacing: var(--tracking-tight);
  line-height: var(--leading-tight);
}
.g-page-header__desc {
  margin-top: var(--space-1);
  font-size: var(--text-sm);
  color: var(--text-tertiary);
  line-height: var(--leading-relaxed);
}
.g-page-header__actions {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  flex-shrink: 0;
}
</style>
