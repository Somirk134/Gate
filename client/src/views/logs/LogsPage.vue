<template>
  <div class="logs-page" @click="closeContextMenu">
    <LogLoading v-if="isLoading" />

    <GCard v-else-if="isError" variant="plain" padding="lg" class="logs-page__error">
      <GErrorState
        :title="t('logs.loadingFailed')"
        :message="error || t('logs.loadingFailedMessage')"
        retry
        @retry="refresh" />
    </GCard>

    <template v-else>
      <header class="logs-shell-header">
        <div>
          <p>{{ t('logs.kicker') }}</p>
          <h1>{{ t('logs.title') }}</h1>
        </div>
        <div class="logs-shell-header__meta">
          <span :class="{ active: autoScroll && !paused }">{{ t('logs.autoScroll') }}</span>
          <span :class="{ active: paused }">{{ paused ? t('logs.paused') : t('logs.live') }}</span>
          <strong>{{ filteredLogs.length }} / {{ logs.length }}</strong>
        </div>
      </header>

      <LogToolbar
        :filter="filter"
        :paused="paused"
        :auto-scroll="autoScroll"
        @update:filter="setFilter"
        @toggle-auto-scroll="setAutoScroll(!autoScroll)"
        @pause="pause"
        @resume="resume"
        @clear="clear"
        @export="exportDialogVisible = true"
        @copy-all="copyAll"
        @refresh="refresh" />

      <LogStatistics :statistics="filteredStatistics" />

      <div v-if="hasLogs" class="logs-workspace">
        <LogSourceTree
          :sources="sourceTree"
          :selected="selectedSource"
          :counts="sourceCounts"
          :total="logs.length"
          @select="selectSource" />

        <LogConsole
          ref="consoleRef"
          :logs="filteredLogs"
          :keyword="filter.keyword"
          :selected-id="selectedId"
          :group-by="filter.groupBy"
          :auto-scroll="autoScroll && !paused"
          @select="selectLog"
          @contextmenu-log="openContextMenu" />

        <LogInspector :log="selectedLog" />
      </div>

      <LogEmpty v-else />

      <LogStatusBar
        :total="logs.length"
        :filtered="filteredLogs.length"
        :paused="paused"
        :auto-scroll="autoScroll"
        :dropped="droppedCount"
        :selected="selectedLog?.id" />
    </template>

    <LogExportDialog
      :visible="exportDialogVisible"
      :count="filteredLogs.length"
      @close="exportDialogVisible = false"
      @export="handleExport" />

    <LogContextMenu
      :visible="contextMenu.visible"
      :x="contextMenu.x"
      :y="contextMenu.y"
      @copy="copySelectedLine"
      @copy-message="copySelectedMessage"
      @copy-json="copySelectedJson"
      @details="focusContextLog"
      @delete="deleteContextLog"
      @clear="clear"
      @export="exportDialogVisible = true" />
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, reactive, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useFeedback } from '@composables/useFeedback'
import GCard from '@components/base/GCard.vue'
import GErrorState from '@components/feedback/GErrorState.vue'
import LogConsole from './components/LogConsole.vue'
import LogContextMenu from './components/LogContextMenu.vue'
import LogEmpty from './components/LogEmpty.vue'
import LogExportDialog from './components/LogExportDialog.vue'
import LogInspector from './components/LogInspector.vue'
import LogLoading from './components/LogLoading.vue'
import LogSourceTree from './components/LogSourceTree.vue'
import LogStatistics from './components/LogStatistics.vue'
import LogStatusBar from './components/LogStatusBar.vue'
import LogToolbar from './components/LogToolbar.vue'
import { useLog, useLogExport } from './hooks'
import { LOG_SOURCE_LIST, LOG_SOURCES } from './constants'
import type { LogFilter, LogItem, LogSource } from './types'
import type { LogExportFormat } from './utils'
import { serializeLogs } from './utils'
import './styles/log.css'

const { toast } = useFeedback()
const { t } = useI18n()
const {
  logs,
  filter,
  paused,
  autoScroll,
  selectedId,
  droppedCount,
  error,
  isLoading,
  isError,
  hasLogs,
  filteredLogs,
  selectedLog,
  filteredStatistics,
  refresh,
  clear,
  remove,
  select,
  setFilter,
  pause,
  resume,
  setAutoScroll,
} = useLog()

const consoleRef = ref<InstanceType<typeof LogConsole> | null>(null)
const exportDialogVisible = ref(false)
const sourceTree = LOG_SOURCES
const contextMenu = reactive<{
  visible: boolean
  x: number
  y: number
  log: LogItem | null
}>({
  visible: false,
  x: 0,
  y: 0,
  log: null,
})

const selectedSource = computed<LogSource | 'ALL'>(() => filter.value.sources[0] ?? 'ALL')
const { exportLogs } = useLogExport(filteredLogs)

function handleRuntimeLogsCleared() {
  clear()
}

onMounted(() => {
  window.addEventListener('gate:logs:cleared', handleRuntimeLogsCleared)
})

onUnmounted(() => {
  window.removeEventListener('gate:logs:cleared', handleRuntimeLogsCleared)
})

const sourceCounts = computed<Record<LogSource, number>>(() => {
  const counts = Object.fromEntries(LOG_SOURCE_LIST.map((source) => [source, 0])) as Record<
    LogSource,
    number
  >
  for (const log of logs.value) counts[log.source] += 1
  return counts
})

watch(
  filteredLogs,
  (items) => {
    if (!selectedId.value && items.length) {
      select(items[items.length - 1].id)
    }
  },
  { immediate: true },
)

function selectSource(source: LogSource | 'ALL') {
  setFilter({ sources: source === 'ALL' ? [] : [source] })
}

function selectLog(log: LogItem) {
  select(log.id)
}

function openContextMenu(event: MouseEvent, log: LogItem) {
  selectLog(log)
  contextMenu.visible = true
  contextMenu.x = Math.min(event.clientX, window.innerWidth - 190)
  contextMenu.y = Math.min(event.clientY, window.innerHeight - 260)
  contextMenu.log = log
}

function closeContextMenu() {
  contextMenu.visible = false
}

async function copyText(text: string, message: string) {
  await navigator.clipboard.writeText(text)
  toast.success(message)
  closeContextMenu()
}

async function copyAll() {
  await copyText(serializeLogs(filteredLogs.value, 'txt'), t('logs.copied'))
}

async function copySelectedLine() {
  const log = contextMenu.log
  if (!log) return
  await copyText(serializeLogs([log], 'txt'), t('logs.lineCopied'))
}

async function copySelectedMessage() {
  const log = contextMenu.log
  if (!log) return
  await copyText(log.message, t('logs.messageCopied'))
}

async function copySelectedJson() {
  const log = contextMenu.log
  if (!log) return
  await copyText(log.raw, t('logs.jsonCopied'))
}

function focusContextLog() {
  if (contextMenu.log) select(contextMenu.log.id)
  closeContextMenu()
}

function deleteContextLog() {
  if (!contextMenu.log) return
  remove(contextMenu.log.id)
  toast.success(t('logs.deleted'))
  closeContextMenu()
}

function handleExport(format: LogExportFormat) {
  exportLogs(format)
  exportDialogVisible.value = false
  toast.success(t('logs.exported'))
}

watch(
  filter,
  (value: LogFilter) => {
    if (value.keyword || value.levels.length || value.sources.length) {
      setAutoScroll(false)
    }
  },
  { deep: true },
)
</script>
