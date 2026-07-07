<template>
  <div class="log-filter">
    <div class="log-filter__levels">
      <button
        v-for="item in levelOptions"
        :key="item.level"
        type="button"
        class="log-filter__chip"
        :class="{ 'log-filter__chip--active': filter.levels.includes(item.level) }"
        :style="{ '--level-color': item.color }"
        @click="toggleLevel(item.level)"
      >
        {{ item.level }}
      </button>
    </div>

    <select class="log-select" :value="primarySource" @change="setSource(($event.target as HTMLSelectElement).value)">
      <option value="">All Sources</option>
      <option v-for="source in sources" :key="source" :value="source">
        {{ sourceLabels[source] }}
      </option>
    </select>

    <select class="log-select" :value="filter.timeRange" @change="patch({ timeRange: ($event.target as HTMLSelectElement).value as LogTimeRange })">
      <option value="all">Any Time</option>
      <option value="15m">Last 15 min</option>
      <option value="1h">Last hour</option>
      <option value="24h">Last 24 hours</option>
      <option value="today">Today</option>
    </select>

    <select class="log-select" :value="filter.groupBy" @change="patch({ groupBy: ($event.target as HTMLSelectElement).value as LogGroupBy })">
      <option value="none">No Group</option>
      <option value="time">Group by Time</option>
      <option value="source">Group by Source</option>
      <option value="level">Group by Level</option>
    </select>

    <button type="button" class="log-filter__toggle" :class="{ 'log-filter__toggle--active': filter.fuzzy }" @click="patch({ fuzzy: !filter.fuzzy })">
      Fuzzy
    </button>
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue"
import type { LogFilter as LogFilterState, LogGroupBy, LogLevel, LogSource, LogTimeRange } from "../types"
import { LOG_LEVELS, LOG_SOURCE_LABELS, LOG_SOURCE_LIST } from "../constants"

const props = defineProps<{
  filter: LogFilterState
}>()

const emit = defineEmits<{ "update:filter": [value: LogFilterState] }>()

const levelOptions = LOG_LEVELS
const sources = LOG_SOURCE_LIST
const sourceLabels = LOG_SOURCE_LABELS
const primarySource = computed(() => props.filter.sources[0] ?? "")

function patch(value: Partial<LogFilterState>) {
  emit("update:filter", { ...props.filter, ...value })
}

function toggleLevel(level: LogLevel) {
  const next = new Set(props.filter.levels)
  if (next.has(level)) next.delete(level)
  else next.add(level)
  patch({ levels: Array.from(next) })
}

function setSource(value: string) {
  patch({ sources: value ? [value as LogSource] : [] })
}
</script>
