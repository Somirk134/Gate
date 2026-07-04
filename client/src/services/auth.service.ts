import { api } from '@api'

export const authService = {
    async login(username: string, password: string) {
        return api.post('/auth/login', { username, password })
    },

    async register(username: string, email: string, password: string) {
        return api.post('/auth/register', { username, email, password })
    },

    async logout() {
        return api.post('/auth/logout')
    },
}
