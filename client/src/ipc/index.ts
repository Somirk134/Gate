import { invoke } from '@tauri-apps/api/core'

export const ipc = {
    connect: (serverAddr: string, token: string) =>
        invoke<string>('connect', { serverAddr, token }),

    disconnect: () => invoke<void>('disconnect'),

    createTunnel: (localPort: number, remotePort: number, protocol: string) =>
        invoke<string>('create_tunnel', { localPort, remotePort, protocol }),

    deleteTunnel: (tunnelId: string) =>
        invoke<void>('delete_tunnel', { tunnelId }),

    getConfig: () => invoke<string>('get_config'),

    setConfig: (key: string, value: string) =>
        invoke<void>('set_config', { key, value }),
}
