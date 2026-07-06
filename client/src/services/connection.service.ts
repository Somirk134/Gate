import { TauriIpcClient } from "@/ipc"
import type { Statistics } from "@/monitoring/types"

const ipc = new TauriIpcClient()

export const connectionService = {
    async list() {
        const statistics = await ipc.invoke<Statistics>("runtime_get_statistics")
        return {
            current: statistics.connection.currentConnection,
            total: statistics.connection.totalConnection,
            reconnect: statistics.connection.reconnect,
            disconnect: statistics.connection.disconnect,
            averageRttMs: statistics.connection.averageRttMs,
        }
    },

    async disconnect(_id?: string) {
        return ipc.invoke<void>("disconnect")
    },

    async heartbeat() {
        return ipc.invoke<number>("heartbeat")
    },
}
