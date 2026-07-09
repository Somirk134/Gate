<!--
  GErrorState — 错误状态
  ------------------------------------------------------------------
  用途：加载失败/请求出错时的占位与重试入口。
-->
<template>
  <div class="g-error-state">
    <div class="g-error-state__icon">
      <GIcon
        name="alert-circle"
        :size="32"
      />
    </div>
    <div
      v-if="title"
      class="g-error-state__title"
    >
      {{ title }}
    </div>
    <div class="g-error-state__desc">
      <slot>{{ message }}</slot>
    </div>
    <div
      v-if="$slots.action || retry"
      class="g-error-state__action"
    >
      <slot name="action">
        <GButton
          v-if="retry"
          variant="secondary"
          size="sm"
          icon="refresh"
          @click="emit('retry')"
        >
          重试
        </GButton>
      </slot>
    </div>
  </div>
</template>

<script setup lang="ts">
import GIcon from "@components/icons/GIcon.vue"
import GButton from "@components/base/GButton.vue"

defineProps<{
  title?: string
  message?: string
  retry?: boolean
}>()

const emit = defineEmits<{ retry: [] }>()
</script>

<style scoped>
.g-error-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: var(--space-2);
  padding: var(--space-10) var(--space-6);
  text-align: center;
}
.g-error-state__icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 56px;
  height: 56px;
  border-radius: var(--radius-full);
  background: var(--color-error-muted);
  color: var(--color-error);
  margin-bottom: var(--space-2);
}
.g-error-state__title {
  font-size: var(--text-md);
  font-weight: var(--weight-medium);
  color: var(--text-primary);
}
.g-error-state__desc {
  font-size: var(--text-sm);
  color: var(--text-tertiary);
  max-width: 320px;
  line-height: var(--leading-relaxed);
}
.g-error-state__action {
  margin-top: var(--space-3);
}
</style>
