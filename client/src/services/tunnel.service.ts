import { TauriIpcClient } from "@/ipc"
import type { DashboardData } from "@/monitoring/types"

const ipc = new TauriIpcClient()

export const tunnelService = {
    async list() {
        const dashboard = await ipc.invoke<DashboardData>("runtime_get_dashboard")
        return dashboard.tunnels
    },

    async create(data: {
        localPort: number
        remotePort: number
        protocol: string
        localHost?: string
        host?: string
        path?: string
    }) {
        return ipc.invoke<string>("create_tunnel", {
            localPort: data.localPort,
            remotePort: data.remotePort,
            protocol: data.protocol,
            localHost: data.localHost,
            host: data.host,
            path: data.path,
        })
    },

    async start(id: string) {
        return ipc.invoke<void>("start_tunnel", { tunnelId: id })
    },

    async stop(id: string) {
        return ipc.invoke<void>("stop_tunnel", { tunnelId: id })
    },

    async restart(id: string) {
        return ipc.invoke<void>("restart_tunnel", { tunnelId: id })
    },

    async edit(
        id: string,
        patch: Partial<{
            name: string
            protocol: string
            localHost: string
            localPort: number
            remotePort: number
            host: string
            path: string
        }>,
    ) {
        return ipc.invoke<void>("edit_tunnel", { tunnelId: id, patch })
    },

    async delete(id: string) {
        return ipc.invoke<void>("delete_tunnel", { tunnelId: id })
    },
}
