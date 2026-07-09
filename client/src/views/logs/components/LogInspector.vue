<template>
  <aside class="log-inspector">
    <template v-if="log">
      <header class="log-inspector__header">
        <div>
          <span class="log-inspector__eyebrow"
            >{{ sourceLabel(log.source) }} / {{ log.module }}</span
          >
          <h2>{{ log.level }}</h2>
        </div>
        <button type="button" class="log-inspector__level" :class="`is-${log.level.toLowerCase()}`">
          {{ log.level }}
        </button>
      </header>

      <div class="log-inspector__message">
        {{ log.message }}
      </div>

      <div class="log-inspector__tabs">
        <button
          v-for="tab in tabs"
          :key="tab.key"
          type="button"
          :class="{ active: activeTab === tab.key }"
          @click="activeTab = tab.key">
          {{ tab.label }}
        </button>
      </div>

      <section v-if="activeTab === 'context'" class="log-inspector__section">
        <dl>
          <template v-for="item in contextItems" :key="item.label">
            <dt>{{ item.label }}</dt>
            <dd>{{ item.value }}</dd>
          </template>
        </dl>
      </section>

      <section v-else-if="activeTab === 'metadata'" class="log-inspector__section">
        <pre>{{ JSON.stringify(log.metadata, null, 2) }}</pre>
      </section>

      <section v-else-if="activeTab === 'stack'" class="log-inspector__section">
        <pre>{{ stackText }}</pre>
      </section>

      <section v-else class="log-inspector__section">
        <pre>{{ log.raw }}</pre>
      </section>
    </template>

    <div v-else class="log-inspector__empty">
      <GIcon name="panel-right-open" :size="28" />
      <span>{{ t('logs.noSelection') }}</span>
    </div>
  </aside>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import GIcon from '@components/icons/GIcon.vue'
import type { LogItem, LogSource } from '../types'
import { formatLogTime } from '../utils'

type TabKey = 'context' | 'metadata' | 'stack' | 'raw'

const props = defineProps<{ log: LogItem | null }>()
const { t } = useI18n()
const activeTab = ref<TabKey>('context')
const tabs = computed<Array<{ key: TabKey; label: string }>>(() => [
  { key: 'context', label: t('logs.tabs.context') },
  { key: 'metadata', label: t('logs.tabs.metadata') },
  { key: 'stack', label: t('logs.tabs.stack') },
  { key: 'raw', label: t('logs.tabs.raw') },
])

const contextItems = computed(() => {
  if (!props.log) return []
  return [
    { label: t('logs.fields.time'), value: formatLogTime(props.log.timestamp) },
    { label: t('logs.fields.traceId'), value: props.log.traceId ?? '-' },
    { label: t('logs.fields.requestId'), value: props.log.requestId ?? '-' },
    { label: t('logs.fields.project'), value: props.log.projectName ?? '-' },
    { label: t('logs.fields.tunnel'), value: props.log.tunnelName ?? '-' },
    { label: t('logs.fields.host'), value: props.log.context.host },
    { label: t('logs.fields.thread'), value: props.log.context.thread },
    { label: t('logs.fields.session'), value: props.log.context.sessionId },
  ]
})

const stackText = computed(() => props.log?.stack?.join('\n') ?? t('logs.stackReserved'))

function sourceLabel(source: LogSource): string {
  return t(`logs.source.${source.toLowerCase()}`)
}

watch(
  () => props.log?.id,
  () => {
    activeTab.value = 'context'
  },
)
</script>
