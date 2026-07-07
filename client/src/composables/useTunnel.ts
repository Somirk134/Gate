import { computed, ref } from "vue"
import { tunnelService } from "@/services/tunnel.service"

export function useTunnel() {
    const tunnels = ref<Awaited<ReturnType<typeof tunnelService.list>>>([])
    const loading = ref(false)
    const error = ref("")

    async function fetchTunnels() {
        loading.value = true
        error.value = ""
        try {
            tunnels.value = await tunnelService.list()
        } catch (err) {
            error.value = err instanceof Error ? err.message : "Failed to load tunnels"
        } finally {
            loading.value = false
        }
    }

    async function createTunnel(localPort: number, remotePort: number, protocol: string) {
        const id = await tunnelService.create({ localPort, remotePort, protocol })
        await fetchTunnels()
        return id
    }

    async function deleteTunnel(id: string) {
        await tunnelService.delete(id)
        await fetchTunnels()
    }

    return {
        tunnels: computed(() => tunnels.value),
        loading: computed(() => loading.value),
        error: computed(() => error.value),
        createTunnel,
        deleteTunnel,
        fetchTunnels,
    }
}
