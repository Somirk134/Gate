<!--
  GActionCard — 快捷操作卡片
  ------------------------------------------------------------------
  用途：仪表盘“新建/快速隧道”等快捷入口。竖排图标+文字+快捷键。
  基于 GCard interactive，点击触发 action。
-->
<template>
  <GCard
    variant="interactive"
    padding="md"
    clickable
    @click="$emit('click')"
  >
    <div class="g-action-card">
      <span class="g-action-card__icon">
        <GIcon
          :name="icon"
          :size="24"
        />
      </span>
      <span class="g-action-card__label">{{ label }}</span>
      <span
        v-if="shortcut"
        class="g-action-card__shortcut"
      >{{ shortcut }}</span>
    </div>
  </GCard>
</template>

<script setup lang="ts">
import GCard from "@components/base/GCard.vue"
import GIcon from "@components/icons/GIcon.vue"

defineProps<{
  icon: string
  label: string
  shortcut?: string
}>()

defineEmits<{ click: [] }>()
</script>

<style scoped>
.g-action-card {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-2);
  text-align: center;
}
.g-action-card__icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: var(--color-primary);
}
.g-action-card__label {
  font-size: var(--text-sm);
  font-weight: var(--weight-medium);
  color: var(--text-primary);
}
.g-action-card__shortcut {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  font-family: var(--font-mono);
  padding: 1px var(--space-2);
  background: var(--bg-input);
  border-radius: var(--radius-sm);
}
</style>
