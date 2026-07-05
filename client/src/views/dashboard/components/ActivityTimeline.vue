<!--
  ActivityTimeline — 最近活动时间线
  ------------------------------------------------------------------
  时间线形式展示最近操作，支持图标/颜色/时间/类型。
-->
<template>
  <section class="dashboard-section">
    <GCard variant="plain" padding="none" class="timeline-card">
      <template #header>
        <GSectionHeader icon="history">
          {{ title }}
        </GSectionHeader>
      </template>

      <div v-if="activities.length" class="timeline">
        <div
          v-for="(item, i) in groupedActivities"
          :key="item.id"
          class="timeline__item"
          :class="`stagger-${(i % 6) + 1}`"
        >
          <div class="timeline__indicator">
            <span class="timeline__dot" :class="`timeline__dot--${item.type}`">
              <GIcon :name="activityIcon(item.type)" :size="11" />
            </span>
            <span v-if="i < groupedActivities.length - 1" class="timeline__line" />
          </div>
          <div class="timeline__content">
            <div class="timeline__title-row">
              <span class="timeline__title">{{ item.title }}</span>
              <span class="timeline__time">{{ relativeTime(item.timestamp) }}</span>
            </div>
            <p v-if="item.description" class="timeline__desc">{{ item.description }}</p>
          </div>
        </div>
      </div>

      <GEmptyState v-else title="暂无活动记录" description="你的操作将在此处显示。" />
    </GCard>
  </section>
</template>

<script setup lang="ts">
import { computed } from "vue"
import GCard from "@components/base/GCard.vue"
import GIcon from "@components/icons/GIcon.vue"
import GSectionHeader from "@components/layout/GSectionHeader.vue"
import GEmptyState from "@components/feedback/GEmptyState.vue"
import type { DashboardActivity, ActivityType } from "../types"

const props = withDefaults(
  defineProps<{
    activities: DashboardActivity[]
    title?: string
  }>(),
  {
    title: "最近活动",
  },
)

const groupedActivities = computed(() => props.activities)

function activityIcon(type: ActivityType): string {
  const map: Record<ActivityType, string> = {
    create: "plus",
    start: "play",
    stop: "stop",
    config: "edit",
    connect: "plug",
    update: "refresh",
    delete: "trash",
  }
  return map[type] ?? "circle"
}

function relativeTime(ts: number): string {
  const diff = Date.now() - ts
  const min = 60 * 1000
  const hour = 60 * min
  const day = 24 * hour
  if (diff < min) return "刚刚"
  if (diff < hour) return `${Math.floor(diff / min)} 分钟前`
  if (diff < day) return `${Math.floor(diff / hour)} 小时前`
  if (diff < 2 * day) return "昨天"
  return `${Math.floor(diff / day)} 天前`
}
</script>

<style scoped>
.timeline-card {
  animation: g-slide-in-up var(--duration-base) var(--ease-out) both;
}
.timeline {
  padding: var(--space-3) var(--space-4);
  display: flex;
  flex-direction: column;
}
.timeline__item {
  display: flex;
  gap: var(--space-3);
  padding: var(--space-2) 0;
  animation: g-slide-in-up var(--duration-base) var(--ease-out) both;
}
.timeline__indicator {
  display: flex;
  flex-direction: column;
  align-items: center;
  flex-shrink: 0;
  padding-top: 2px;
}
.timeline__dot {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 22px;
  height: 22px;
  border-radius: var(--radius-full);
  color: #fff;
  flex-shrink: 0;
  z-index: 1;
}
.timeline__dot--create { background: var(--color-primary); }
.timeline__dot--start { background: var(--color-success); }
.timeline__dot--stop { background: var(--status-offline); }
.timeline__dot--config { background: var(--color-warning); }
.timeline__dot--connect { background: var(--color-info); }
.timeline__dot--update { background: var(--color-secondary); }
.timeline__dot--delete { background: var(--color-error); }
.timeline__line {
  width: 2px;
  flex: 1;
  background: var(--border-subtle);
  margin-top: 2px;
  min-height: 16px;
}
.timeline__content {
  flex: 1;
  min-width: 0;
  padding-bottom: var(--space-2);
}
.timeline__title-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-2);
}
.timeline__title {
  font-size: var(--text-sm);
  font-weight: var(--weight-medium);
  color: var(--text-primary);
}
.timeline__time {
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  font-variant-numeric: tabular-nums;
  flex-shrink: 0;
}
.timeline__desc {
  margin-top: 2px;
  font-size: var(--text-xs);
  color: var(--text-tertiary);
  line-height: var(--leading-relaxed);
}
</style>
