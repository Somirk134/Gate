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
  const error = ref('')

  async function fetchConnections() {
    loading.value = true
    error.value = ''
    connections.value = []
    loading.value = false
  }

  return {
    connections,
    loading,
    error,
    fetchConnections,
  }
})
