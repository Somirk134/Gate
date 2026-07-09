<template>
  <div class="settings-empty">
    <GIcon name="search" :size="34" />
    <h2>{{ t('settings.legacy.emptyTitle') }}</h2>
    <p v-if="query">{{ t('settings.legacy.emptyWithQuery', { query }) }}</p>
    <p v-else>{{ t('settings.legacy.emptyByCategory') }}</p>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import GIcon from '@components/icons/GIcon.vue'

withDefaults(
  defineProps<{
    query?: string
  }>(),
  {
    query: '',
  },
)

const { t } = useI18n()
</script>
