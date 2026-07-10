<template>
  <aside class="runtime-activity-feed">
    <header>
      <div>
        <span>Activity Feed</span>
        <strong>Realtime Log</strong>
      </div>
      <button type="button" @click="paused = !paused">
        <GIcon :name="paused ? 'play' : 'pause'" :size="14" />
        {{ paused ? 'Resume' : 'Pause' }}
      </button>
    </header>

    <div ref="listRef" class="runtime-activity-feed__list">
      <article v-for="item in visibleItems" :key="item.id">
        <time>{{ formatTime(item.timestamp) }}</time>
        <div>
          <strong>{{ item.title }}</strong>
          <span>{{ item.category }}</span>
        </div>
      </article>
      <div v-if="!visibleItems.length" class="runtime-activity-feed__empty">
        <GIcon name="logs" :size="20" />
        <span>No runtime activity</span>
      </div>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import GIcon from '@components/icons/GIcon.vue'
import type { RecentActivity } from '@/monitoring/types'

const props = withDefaults(
  defineProps<{
    items: RecentActivity[]
    max?: number
  }>(),
  {
    max: 18,
  },
)

const paused = ref(false)
const listRef = ref<HTMLElement | null>(null)
const visibleItems = computed(() =>
  [...props.items].sort((left, right) => right.timestamp - left.timestamp).slice(0, props.max),
)

watch(
  () => visibleItems.value.map((item) => item.id).join('|'),
  async () => {
    if (paused.value) return
    await nextTick()
    if (listRef.value) {
      listRef.value.scrollTop = 0
    }
  },
)

function formatTime(timestamp: number) {
  return new Intl.DateTimeFormat('zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  }).format(timestamp)
}
</script>

<style scoped>
.runtime-activity-feed {
  min-width: 0;
  min-height: 0;
  display: grid;
  grid-template-rows: auto minmax(0, 1fr);
  border: 1px solid var(--border-subtle);
  border-radius: 8px;
  background: var(--bg-surface);
}

.runtime-activity-feed header {
  min-height: 52px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  padding: 0 var(--space-3);
  border-bottom: 1px solid var(--border-subtle);
}

.runtime-activity-feed header span {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

.runtime-activity-feed header strong {
  display: block;
  color: var(--text-primary);
  font-size: var(--text-sm);
}

.runtime-activity-feed button {
  height: 28px;
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  border: 1px solid var(--border-default);
  border-radius: 8px;
  background: var(--bg-input);
  color: var(--text-secondary);
  cursor: pointer;
  font-size: var(--text-xs);
}

.runtime-activity-feed__list {
  min-height: 0;
  overflow: auto;
  padding: var(--space-2);
}

.runtime-activity-feed__list article {
  display: grid;
  grid-template-columns: 56px minmax(0, 1fr);
  gap: var(--space-2);
  padding: var(--space-2);
  border-radius: 8px;
}

.runtime-activity-feed__list article:hover {
  background: var(--bg-surface-hover);
}

.runtime-activity-feed time {
  color: var(--text-tertiary);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
}

.runtime-activity-feed strong,
.runtime-activity-feed span {
  display: block;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.runtime-activity-feed strong {
  color: var(--text-primary);
  font-size: var(--text-xs);
}

.runtime-activity-feed article span {
  margin-top: 2px;
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

.runtime-activity-feed__empty {
  min-height: 180px;
  display: grid;
  place-content: center;
  justify-items: center;
  gap: var(--space-2);
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}
</style>
