import { TauriIpcClient } from "@/ipc"
import type { DashboardData } from "@/monitoring/types"

const ipc = new TauriIpcClient()

export const tunnelService = {
    async list() {
        const dashboard = await ipc.invoke<DashboardData>("runtime_get_dashboard")
        return dashboard.tunnels
    },

    async create(data: { localPort: number; remotePort: number; protocol: string }) {
        return ipc.invoke<string>("create_tunnel", {
            localPort: data.localPort,
            remotePort: data.remotePort,
            protocol: data.protocol,
        })
    },

    async delete(id: string) {
        return ipc.invoke<void>("delete_tunnel", { tunnelId: id })
    },
}
