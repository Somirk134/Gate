<!--
  GEmptyState — 空状态
  ------------------------------------------------------------------
  用途：列表/卡片无数据时的占位。统一图标 + 标题 + 描述 + 操作。
  Slots:
    icon     自定义图标（默认空盒）
    default  描述文字
    action   操作按钮区
-->
<template>
  <div class="g-empty-state">
    <div class="g-empty-state__icon">
      <slot name="icon">
        <GIcon name="inbox" :size="32" />
      </slot>
    </div>
    <div v-if="title" class="g-empty-state__title">{{ title }}</div>
    <div v-if="$slots.default || description" class="g-empty-state__desc">
      <slot>{{ description }}</slot>
    </div>
    <div v-if="$slots.action" class="g-empty-state__action">
      <slot name="action" />
    </div>
  </div>
</template>

<script setup lang="ts">
import GIcon from "@components/icons/GIcon.vue"

// inbox 图标占位：若 registry 无则回退 info。补充到 registry 可省此处判断
defineProps<{
  title?: string
  description?: string
}>()
</script>

<style scoped>
.g-empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--space-2);
  padding: var(--space-10) var(--space-6);
  text-align: center;
}
.g-empty-state__icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 56px;
  height: 56px;
  border-radius: var(--radius-full);
  background: var(--bg-surface-hover);
  color: var(--text-tertiary);
  margin-bottom: var(--space-2);
}
.g-empty-state__title {
  font-size: var(--text-md);
  font-weight: var(--weight-medium);
  color: var(--text-secondary);
}
.g-empty-state__desc {
  font-size: var(--text-sm);
  color: var(--text-tertiary);
  max-width: 320px;
  line-height: var(--leading-relaxed);
}
.g-empty-state__action {
  margin-top: var(--space-3);
}
</style>
