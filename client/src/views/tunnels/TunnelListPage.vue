<template>
    <n-space vertical>
        <n-h1>Tunnels</n-h1>
        <n-data-table :columns="columns" :data="tunnels" :loading="loading" />
    </n-space>
</template>

<script setup lang="ts">
import { ref } from 'vue'

const loading = ref(false)
const tunnels = ref([])

const columns = [
    { title: 'ID', key: 'id' },
    { title: 'Local Port', key: 'localPort' },
    { title: 'Remote Port', key: 'remotePort' },
    { title: 'Protocol', key: 'protocol' },
    { title: 'Status', key: 'status' },
]
</script>
