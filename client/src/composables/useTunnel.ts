import { useTunnelStore } from '@stores'

export function useTunnel() {
    const tunnelStore = useTunnelStore()

    function createTunnel(_localPort: number, _remotePort: number, _protocol: string) {
        // TODO: implement
    }

    function deleteTunnel(_id: string) {
        // TODO: implement
    }

    return {
        tunnels: tunnelStore.tunnels,
        loading: tunnelStore.loading,
        createTunnel,
        deleteTunnel,
        fetchTunnels: tunnelStore.fetchTunnels,
    }
}
