<template>
  <div v-if="visible" class="log-dialog-backdrop">
    <section class="log-export-dialog">
      <header>
        <h2>{{ t('logs.exportLogs') }}</h2>
        <button type="button" @click="$emit('close')">
          <GIcon name="close" :size="16" />
        </button>
      </header>
      <p>{{ t('logs.exportHint', { count }) }}</p>
      <div class="log-export-dialog__formats">
        <button
          v-for="item in formats"
          :key="item.value"
          type="button"
          :class="{ active: format === item.value }"
          @click="format = item.value">
          <GIcon :name="item.icon" :size="16" />
          <span>{{ item.label }}</span>
        </button>
      </div>
      <footer>
        <GButton variant="secondary" @click="$emit('close')">
          {{ t('common.cancel') }}
        </GButton>
        <GButton variant="primary" icon="download" @click="$emit('export', format)">
          {{ t('logs.export') }}
        </GButton>
      </footer>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import GButton from '@components/base/GButton.vue'
import GIcon from '@components/icons/GIcon.vue'
import type { LogExportFormat } from '../utils'

defineProps<{
  visible: boolean
  count: number
}>()

defineEmits<{
  close: []
  export: [format: LogExportFormat]
}>()

const format = ref<LogExportFormat>('json')
const { t } = useI18n()
const formats = computed<Array<{ value: LogExportFormat; label: string; icon: string }>>(() => [
  { value: 'json', label: 'JSON', icon: 'file-code' },
  { value: 'ndjson', label: 'NDJSON', icon: 'code' },
  { value: 'txt', label: t('logs.textFormat'), icon: 'file-text' },
])
</script>
