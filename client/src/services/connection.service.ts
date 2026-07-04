import { api } from '@api'

export const connectionService = {
    async list() {
        return api.get('/connections')
    },

    async disconnect(id: string) {
        return api.post(`/connections/${id}/disconnect`)
    },
}
