import { defineStore } from 'pinia'
import { ref } from 'vue'
import { i18n } from '@/i18n'
import { tunnelService } from '@/services/tunnel.service'

export interface Tunnel {
  id: string
  localPort: number
  remotePort: number
  protocol: string
  status: string
}

function t(key: string, params?: Record<string, unknown>): string {
  return (i18n.global as unknown as { t: (key: string, params?: Record<string, unknown>) => string }).t(
    key,
    params,
  )
}

export const useTunnelStore = defineStore('tunnel', () => {
  const tunnels = ref<Tunnel[]>([])
  const loading = ref(false)
  const error = ref('')

  async function fetchTunnels() {
    loading.value = true
    error.value = ''
    try {
      tunnels.value = (await tunnelService.list()).map((tunnel) => ({
        id: tunnel.id,
        localPort: tunnel.localPort ?? 0,
        remotePort: tunnel.remotePort ?? 0,
        protocol: tunnel.protocol,
        status: tunnel.status,
      }))
    } catch (err) {
      error.value = err instanceof Error ? err.message : t('tunnel.errors.loadFailed')
    } finally {
      loading.value = false
    }
  }

  async function addTunnel(tunnel: Tunnel) {
    await tunnelService.create({
      localPort: tunnel.localPort,
      remotePort: tunnel.remotePort,
      protocol: tunnel.protocol,
    })
    await fetchTunnels()
  }

  async function removeTunnel(id: string) {
    await tunnelService.delete(id)
    await fetchTunnels()
  }

  return {
    tunnels,
    loading,
    error,
    fetchTunnels,
    addTunnel,
    removeTunnel,
  }
})
