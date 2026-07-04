<template>
    <n-space vertical>
        <n-h1>Connections</n-h1>
        <n-data-table :columns="columns" :data="connections" :loading="loading" />
    </n-space>
</template>

<script setup lang="ts">
import { ref } from 'vue'

const loading = ref(false)
const connections = ref([])

const columns = [
    { title: 'ID', key: 'id' },
    { title: 'Client', key: 'clientId' },
    { title: 'Remote Address', key: 'remoteAddr' },
    { title: 'Connected At', key: 'connectedAt' },
]
</script>
