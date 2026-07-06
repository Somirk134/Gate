import { TauriIpcClient } from "@/ipc"
import type { DashboardData } from "@/monitoring/types"

const ipc = new TauriIpcClient()

export const projectService = {
    async list() {
        const dashboard = await ipc.invoke<DashboardData>("runtime_get_dashboard")
        return {
            currentWorkspace: dashboard.statistics.client.currentWorkspace,
            openProject: dashboard.statistics.client.openProject,
            activeProject: dashboard.overview.runningTunnel,
        }
    },
}
