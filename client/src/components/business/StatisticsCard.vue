<!--
  StatisticsCard — 统计区块卡片（业务组件模板）
  ------------------------------------------------------------------
  用途：将多个 GStatCard 组合在一组，带区块标题。
  属业务组件：基于 GCard + GStatCard 组合，无逻辑。
  复用：GCard / GStatCard / GSectionHeader
-->
<template>
  <GCard
    variant="plain"
    padding="none"
  >
    <template #header>
      <GSectionHeader :icon="icon">
        {{ title }}
      </GSectionHeader>
    </template>

    <div class="statistics-card__grid">
      <GStatCard
        v-for="item in stats"
        :key="item.label"
        :label="item.label"
        :value="item.value"
        :icon="item.icon"
        :variant="item.variant ?? 'neutral'"
        :trend="item.trend"
      />
    </div>
  </GCard>
</template>

<script setup lang="ts">
import GCard from "@components/base/GCard.vue"
import GStatCard from "@components/cards/GStatCard.vue"
import GSectionHeader from "@components/layout/GSectionHeader.vue"

defineProps<{
  title: string
  icon?: string
  stats: Array<{
    label: string
    value: string | number
    icon?: string
    variant?: "neutral" | "primary" | "success" | "warning" | "error" | "info"
    trend?: string
  }>
}>()
</script>

<style scoped>
.statistics-card__grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
  gap: var(--space-3);
  padding: var(--space-4);
}
</style>
