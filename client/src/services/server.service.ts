import { TauriIpcClient } from "@/ipc"
import type { DashboardData, HealthReport } from "@/monitoring/types"

const ipc = new TauriIpcClient()

export const serverService = {
    async status() {
        const dashboard = await ipc.invoke<DashboardData>("runtime_get_dashboard")
        return dashboard.serverStatus
    },

    async health() {
        return ipc.invoke<HealthReport>("runtime_get_health")
    },
}
