<template>
  <n-space vertical>
    <n-h1>{{ t('connections.title') }}</n-h1>
    <n-data-table
      :columns="columns"
      :data="connections"
      :loading="loading"
    />
  </n-space>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useI18n } from 'vue-i18n'

const { t } = useI18n()

const loading = ref(false)
const connections = ref([])

const columns = computed(() => [
    { title: t('common.details'), key: 'id' },
    { title: t('connections.client'), key: 'clientId' },
    { title: t('connections.remoteAddr'), key: 'remoteAddr' },
    { title: t('connections.connectedAt'), key: 'connectedAt' },
])
</script>
