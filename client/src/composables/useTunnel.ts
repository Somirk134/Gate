import { useTunnelStore } from '@stores'

export function useTunnel() {
    const tunnelStore = useTunnelStore()

    function createTunnel(localPort: number, remotePort: number, protocol: string) {
        todo
    }

    function deleteTunnel(id: string) {
        todo
    }

    return {
        tunnels: tunnelStore.tunnels,
        loading: tunnelStore.loading,
        createTunnel,
        deleteTunnel,
        fetchTunnels: tunnelStore.fetchTunnels,
    }
}
