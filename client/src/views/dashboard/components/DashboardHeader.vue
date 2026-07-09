<!--
  DashboardHeader — Dashboard 顶部标题栏
  ------------------------------------------------------------------
  显示页面标题、最后更新时间与刷新按钮。
-->
<template>
  <div class="dashboard-header">
    <div class="dashboard-header__main">
      <h1 class="dashboard-header__title">
        <GIcon name="layout-grid" :size="22" class="dashboard-header__icon" />
        {{ title }}
      </h1>
      <p class="dashboard-header__desc">
        {{ description }}
      </p>
    </div>
    <div class="dashboard-header__actions">
      <span v-if="lastUpdated" class="dashboard-header__updated">
        <GIcon name="clock" :size="12" />
        {{ updatedText }}
      </span>
      <GIconButton
        name="refresh"
        size="sm"
        variant="soft"
        :tooltip="$t('common.loading')"
        @click="$emit('refresh')" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import GIcon from '@components/icons/GIcon.vue'
import GIconButton from '@components/base/GIconButton.vue'

const props = defineProps<{
  title: string
  description: string
  lastUpdated: number
  refreshing?: boolean
}>()

defineEmits<{ refresh: [] }>()

const updatedText = computed(() => {
  if (!props.lastUpdated) return ''
  const diff = Date.now() - props.lastUpdated
  const sec = Math.floor(diff / 1000)
  if (sec < 60) return `${sec} 秒前更新`
  const min = Math.floor(sec / 60)
  if (min < 60) return `${min} 分钟前更新`
  return new Date(props.lastUpdated).toLocaleTimeString()
})
</script>

<style scoped>
.dashboard-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-4);
}
.dashboard-header__main {
  min-width: 0;
}
.dashboard-header__title {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  font-size: var(--text-2xl);
  font-weight: var(--weight-semibold);
  color: var(--text-primary);
  letter-spacing: var(--tracking-tight);
  line-height: var(--leading-tight);
}
.dashboard-header__icon {
  color: var(--color-primary);
}
.dashboard-header__desc {
  margin-top: var(--space-1);
  font-size: var(--text-sm);
  color: var(--text-tertiary);
}
.dashboard-header__actions {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  flex-shrink: 0;
}
.dashboard-header__updated {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  font-variant-numeric: tabular-nums;
}
</style>
