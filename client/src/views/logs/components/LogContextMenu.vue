<template>
  <div
    v-if="visible"
    class="log-context-menu"
    :style="{ left: `${x}px`, top: `${y}px` }"
    @click.stop>
    <button type="button" @click="$emit('copy')">
      <GIcon name="copy" :size="14" />{{ t('logs.copy') }}
    </button>
    <button type="button" @click="$emit('copy-message')">
      <GIcon name="clipboard" :size="14" />{{ t('logs.copyMessage') }}
    </button>
    <button type="button" @click="$emit('copy-json')">
      <GIcon name="file-code" :size="14" />{{ t('logs.copyJson') }}
    </button>
    <button type="button" @click="$emit('details')">
      <GIcon name="info-circle" :size="14" />{{ t('logs.viewDetails') }}
    </button>
    <hr />
    <button type="button" @click="$emit('delete')">
      <GIcon name="trash" :size="14" />{{ t('common.delete') }}
    </button>
    <button type="button" @click="$emit('clear')">
      <GIcon name="close" :size="14" />{{ t('logs.clear') }}
    </button>
    <button type="button" @click="$emit('export')">
      <GIcon name="download" :size="14" />{{ t('logs.export') }}
    </button>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import GIcon from '@components/icons/GIcon.vue'

defineProps<{
  visible: boolean
  x: number
  y: number
}>()

defineEmits<{
  copy: []
  'copy-message': []
  'copy-json': []
  details: []
  delete: []
  clear: []
  export: []
}>()

const { t } = useI18n()
</script>
