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
        @click="toggleLevel(item.level)">
        {{ item.level }}
      </button>
    </div>

    <select
      class="log-select"
      :value="primarySource"
      @change="setSource(($event.target as HTMLSelectElement).value)">
      <option value="">
        {{ t('logs.allSources') }}
      </option>
      <option v-for="source in sources" :key="source" :value="source">
        {{ sourceLabel(source) }}
      </option>
    </select>

    <select
      class="log-select"
      :value="filter.timeRange"
      @change="patch({ timeRange: ($event.target as HTMLSelectElement).value as LogTimeRange })">
      <option value="all">
        {{ t('logs.anyTime') }}
      </option>
      <option value="15m">
        {{ t('logs.last15m') }}
      </option>
      <option value="1h">
        {{ t('logs.lastHour') }}
      </option>
      <option value="24h">
        {{ t('logs.last24h') }}
      </option>
      <option value="today">
        {{ t('logs.today') }}
      </option>
    </select>

    <select
      class="log-select"
      :value="filter.groupBy"
      @change="patch({ groupBy: ($event.target as HTMLSelectElement).value as LogGroupBy })">
      <option value="none">
        {{ t('logs.noGroup') }}
      </option>
      <option value="time">
        {{ t('logs.groupByTime') }}
      </option>
      <option value="source">
        {{ t('logs.groupBySource') }}
      </option>
      <option value="level">
        {{ t('logs.groupByLevel') }}
      </option>
    </select>

    <button
      type="button"
      class="log-filter__toggle"
      :class="{ 'log-filter__toggle--active': filter.fuzzy }"
      @click="patch({ fuzzy: !filter.fuzzy })">
      {{ t('logs.fuzzy') }}
    </button>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import type {
  LogFilter as LogFilterState,
  LogGroupBy,
  LogLevel,
  LogSource,
  LogTimeRange,
} from '../types'
import { LOG_LEVELS, LOG_SOURCE_LIST } from '../constants'

const props = defineProps<{
  filter: LogFilterState
}>()

const emit = defineEmits<{ 'update:filter': [value: LogFilterState] }>()

const { t } = useI18n()
const levelOptions = LOG_LEVELS
const sources = LOG_SOURCE_LIST
const primarySource = computed(() => props.filter.sources[0] ?? '')

function patch(value: Partial<LogFilterState>) {
  emit('update:filter', { ...props.filter, ...value })
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

function sourceLabel(source: LogSource): string {
  return t(`logs.source.${source.toLowerCase()}`)
}
</script>
