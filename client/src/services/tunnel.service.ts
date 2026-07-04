import { api } from '@api'

export const tunnelService = {
    async list() {
        return api.get('/tunnels')
    },

    async create(data: { localPort: number; remotePort: number; protocol: string }) {
        return api.post('/tunnels', data)
    },

    async delete(id: string) {
        return api.delete(`/tunnels/${id}`)
    },
}
