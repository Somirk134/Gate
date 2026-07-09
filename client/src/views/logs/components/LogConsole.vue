<template>
  <main ref="containerRef" class="log-console" @scroll="onScroll">
    <div class="log-console__head">
      <span>{{ t('logs.columns.time') }}</span>
      <span>{{ t('logs.columns.level') }}</span>
      <span>{{ t('logs.columns.source') }}</span>
      <span>{{ t('logs.columns.module') }}</span>
      <span>{{ t('logs.columns.message') }}</span>
      <span>{{ t('logs.columns.traceId') }}</span>
      <span>{{ t('logs.columns.requestId') }}</span>
    </div>

    <div v-if="rows.length" class="log-console__viewport" :style="{ height: `${totalHeight}px` }">
      <div class="log-console__window" :style="{ transform: `translateY(${offsetY}px)` }">
        <template v-for="row in visibleRows" :key="row.key">
          <div v-if="row.kind === 'header'" class="log-console__group">
            <GIcon :name="row.icon" :size="13" />
            <span>{{ row.label }}</span>
            <strong>{{ row.count }}</strong>
          </div>
          <LogLine
            v-else
            :log="row.log"
            :keyword="keyword"
            :active="selectedId === row.log.id"
            @select="$emit('select', $event)"
            @contextmenu="$emit('contextmenu-log', $event, row.log)" />
        </template>
      </div>
    </div>

    <div v-else class="log-console__empty-inline">
      <GIcon name="search" :size="18" />
      <span>{{ t('logs.empty') }}</span>
    </div>
  </main>
</template>

<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import GIcon from '@components/icons/GIcon.vue'
import LogLine from './LogLine.vue'
import type { LogGroupBy, LogItem } from '../types'
import { formatLogDate } from '../utils'

type HeaderRow = {
  kind: 'header'
  key: string
  label: string
  icon: string
  count: number
}
type ItemRow = {
  kind: 'item'
  key: string
  log: LogItem
}
type ConsoleRow = HeaderRow | ItemRow

const props = withDefaults(
  defineProps<{
    logs: LogItem[]
    keyword: string
    selectedId?: string | null
    groupBy?: LogGroupBy
    autoScroll?: boolean
  }>(),
  {
    selectedId: null,
    groupBy: 'none',
    autoScroll: true,
  },
)

defineEmits<{
  select: [log: LogItem]
  'contextmenu-log': [event: MouseEvent, log: LogItem]
}>()

const rowHeight = 28
const overscan = 12
const { t } = useI18n()
const containerRef = ref<HTMLElement | null>(null)
const scrollTop = ref(0)
const viewportHeight = ref(0)

const rows = computed<ConsoleRow[]>(() => {
  if (props.groupBy === 'none') {
    return props.logs.map((log) => ({ kind: 'item', key: log.id, log }))
  }

  const grouped = new Map<string, LogItem[]>()
  for (const log of props.logs) {
    const key =
      props.groupBy === 'time'
        ? formatLogDate(log.timestamp)
        : props.groupBy === 'source'
          ? log.source
          : log.level
    const list = grouped.get(key) ?? []
    list.push(log)
    grouped.set(key, list)
  }

  const output: ConsoleRow[] = []
  for (const [label, items] of grouped.entries()) {
    output.push({
      kind: 'header',
      key: `header-${props.groupBy}-${label}`,
      label: formatGroupLabel(label),
      icon:
        props.groupBy === 'time' ? 'calendar' : props.groupBy === 'source' ? 'layers' : 'filter',
      count: items.length,
    })
    output.push(...items.map((log) => ({ kind: 'item' as const, key: log.id, log })))
  }
  return output
})

const totalHeight = computed(() => rows.value.length * rowHeight)
const startIndex = computed(() => Math.max(0, Math.floor(scrollTop.value / rowHeight) - overscan))
const endIndex = computed(() =>
  Math.min(
    rows.value.length,
    Math.ceil((scrollTop.value + viewportHeight.value) / rowHeight) + overscan,
  ),
)
const visibleRows = computed(() => rows.value.slice(startIndex.value, endIndex.value))
const offsetY = computed(() => startIndex.value * rowHeight)

function formatGroupLabel(label: string): string {
  if (props.groupBy === 'source') return t(`logs.source.${label.toLowerCase()}`)
  if (props.groupBy === 'level') return t(`logs.level.${label.toLowerCase()}`)
  return label
}

function onScroll() {
  if (!containerRef.value) return
  scrollTop.value = containerRef.value.scrollTop
  viewportHeight.value = containerRef.value.clientHeight
}

function clampScrollTop() {
  if (!containerRef.value) return
  const maxTop = Math.max(0, totalHeight.value - containerRef.value.clientHeight)
  if (containerRef.value.scrollTop > maxTop) {
    containerRef.value.scrollTop = maxTop
    onScroll()
  }
}

async function scrollToTop() {
  await nextTick()
  if (!containerRef.value) return
  containerRef.value.scrollTop = 0
  onScroll()
}

watch(
  () => [props.logs.length, props.autoScroll],
  () => {
    if (props.autoScroll) {
      void scrollToTop()
      return
    }
    clampScrollTop()
  },
  { flush: 'post' },
)

watch(containerRef, () => {
  if (!containerRef.value) return
  viewportHeight.value = containerRef.value.clientHeight
  if (props.autoScroll) void scrollToTop()
})

defineExpose({ scrollToTop })
</script>
