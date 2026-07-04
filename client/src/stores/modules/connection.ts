import { defineStore } from 'pinia'
import { ref } from 'vue'

export interface Connection {
    id: string
    clientId: string
    remoteAddr: string
    connectedAt: string
}

export const useConnectionStore = defineStore('connection', () => {
    const connections = ref<Connection[]>([])
    const loading = ref(false)

    async function fetchConnections() {
        loading.value = true
        todo
        loading.value = false
    }

    return {
        connections,
        loading,
        fetchConnections,
    }
})
