<!--
  NewsCard — 资讯（预留）
  ------------------------------------------------------------------
  展示 Release Note / GitHub Release / 更新日志（Mock）。
-->
<template>
  <section class="dashboard-section">
    <GCard variant="plain" padding="none" class="news-card">
      <template #header>
        <GSectionHeader icon="sparkles">
          {{ title }}
        </GSectionHeader>
      </template>

      <div v-if="news.length" class="news__list">
        <button
          v-for="(item, i) in news"
          :key="item.id"
          class="news__item"
          :class="`stagger-${(i % 6) + 1}`"
          @click="$emit('open', item)"
        >
          <span class="news__icon" :class="`news__icon--${item.type}`">
            <GIcon :name="newsIcon(item.type)" :size="14" />
          </span>
          <div class="news__body">
            <div class="news__title-row">
              <span class="news__title">{{ item.title }}</span>
              <GBadge v-if="item.version" variant="primary" type="soft" size="sm">
                {{ item.version }}
              </GBadge>
            </div>
            <p class="news__summary">{{ item.summary }}</p>
            <span class="news__date">{{ item.date }}</span>
          </div>
          <GIcon name="chevron-right" :size="14" class="news__arrow" />
        </button>
      </div>

      <GEmptyState v-else title="暂无资讯" />
    </GCard>
  </section>
</template>

<script setup lang="ts">
import GCard from "@components/base/GCard.vue"
import GBadge from "@components/base/GBadge.vue"
import GIcon from "@components/icons/GIcon.vue"
import GSectionHeader from "@components/layout/GSectionHeader.vue"
import GEmptyState from "@components/feedback/GEmptyState.vue"
import type { DashboardNews } from "../types"

withDefaults(
  defineProps<{
    news: DashboardNews[]
    title?: string
  }>(),
  {
    title: "资讯与更新",
  },
)

defineEmits<{ open: [news: DashboardNews] }>()

function newsIcon(type: DashboardNews["type"]): string {
  switch (type) {
    case "release": return "rocket"
    case "github": return "github"
    case "changelog": return "file-text"
    default: return "info-circle"
  }
}
</script>

<style scoped>
.news-card {
  animation: g-slide-in-up var(--duration-base) var(--ease-out) both;
}
.news__list {
  display: flex;
  flex-direction: column;
  padding: var(--space-2);
}
.news__item {
  display: flex;
  align-items: flex-start;
  gap: var(--space-3);
  padding: var(--space-3);
  border: none;
  background: transparent;
  border-radius: var(--radius-md);
  cursor: pointer;
  text-align: left;
  font-family: var(--font-ui);
  transition: background var(--duration-fast) var(--ease-out);
  animation: g-slide-in-up var(--duration-base) var(--ease-out) both;
}
.news__item:hover {
  background: var(--bg-surface-hover);
}
.news__icon {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: var(--radius-md);
  background: var(--bg-surface-hover);
  color: var(--text-tertiary);
  flex-shrink: 0;
  margin-top: 2px;
}
.news__icon--release {
  background: var(--color-primary-muted);
  color: var(--color-primary);
}
.news__icon--github {
  background: var(--color-info-muted);
  color: var(--color-info);
}
.news__icon--changelog {
  background: var(--color-warning-muted);
  color: var(--color-warning);
}
.news__body {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 3px;
}
.news__title-row {
  display: flex;
  align-items: center;
  gap: var(--space-2);
}
.news__title {
  font-size: var(--text-sm);
  font-weight: var(--weight-medium);
  color: var(--text-primary);
}
.news__summary {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  line-height: var(--leading-relaxed);
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
.news__date {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  font-variant-numeric: tabular-nums;
}
.news__arrow {
  color: var(--text-tertiary);
  flex-shrink: 0;
  margin-top: 4px;
  transition: transform var(--duration-fast) var(--ease-out);
}
.news__item:hover .news__arrow {
  transform: translateX(2px);
  color: var(--text-secondary);
}
</style>
