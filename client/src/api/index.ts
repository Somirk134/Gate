import axios from 'axios'
import { tryGetApplicationContext } from '@/providers/appContext'
import { STORAGE_SERVICE } from '@/services/tokens'

export const api = axios.create({
    baseURL: 'http://localhost:5800/api/v1',
    timeout: 10000,
    headers: {
        'Content-Type': 'application/json',
    },
})

api.interceptors.request.use((config) => {
    const token = tryGetApplicationContext()
        ?.services.optional(STORAGE_SERVICE)
        ?.get<string>('auth_token', { namespace: 'auth', cache: true })
    if (token) {
        config.headers.Authorization = `Bearer ${token}`
    }
    return config
})

api.interceptors.response.use(
    (response) => response,
    (error) => {
        if (error.response?.status === 401) {
            tryGetApplicationContext()
                ?.services.optional(STORAGE_SERVICE)
                ?.remove('auth_token', { namespace: 'auth' })
        }
        return Promise.reject(error)
    },
)
