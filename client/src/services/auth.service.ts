import { TauriIpcClient } from '@/ipc'

const ipc = new TauriIpcClient()

export const authService = {
  async connect(serverAddr: string, token: string) {
    return ipc.invoke<string>('connect', { serverAddr, token })
  },

  async login(username: string, password: string) {
    return ipc.invoke<string>('connect', {
      serverAddr: username,
      token: password,
    })
  },

  async register(username: string, email: string, password: string) {
    return ipc.invoke<string>('set_config', {
      key: `auth.user.${username}`,
      value: JSON.stringify({ email, passwordConfigured: password.length > 0 }),
    })
  },

  async logout() {
    return ipc.invoke<void>('disconnect')
  },
}
