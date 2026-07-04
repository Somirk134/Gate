import { defineStore } from 'pinia'
import { ref } from 'vue'

export interface Tunnel {
    id: string
    localPort: number
    remotePort: number
    protocol: string
    status: string
}

export const useTunnelStore = defineStore('tunnel', () => {
    const tunnels = ref<Tunnel[]>([])
    const loading = ref(false)

    async function fetchTunnels() {
        loading.value = true
        todo
        loading.value = false
    }

    function addTunnel(tunnel: Tunnel) {
        tunnels.value.push(tunnel)
    }

    function removeTunnel(id: string) {
        tunnels.value = tunnels.value.filter((t) => t.id !== id)
    }

    return {
        tunnels,
        loading,
        fetchTunnels,
        addTunnel,
        removeTunnel,
    }
})
