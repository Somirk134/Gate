import { computed, ref } from 'vue'
import { i18n } from '@/i18n'
import { tunnelService } from '@/services/tunnel.service'

function t(key: string, params?: Record<string, unknown>): string {
  return (i18n.global as unknown as { t: (key: string, params?: Record<string, unknown>) => string }).t(
    key,
    params,
  )
}

export function useTunnel() {
  const tunnels = ref<Awaited<ReturnType<typeof tunnelService.list>>>([])
  const loading = ref(false)
  const error = ref('')

  async function fetchTunnels() {
    loading.value = true
    error.value = ''
    try {
      tunnels.value = await tunnelService.list()
    } catch (err) {
      error.value = err instanceof Error ? err.message : t('tunnel.errors.loadFailed')
    } finally {
      loading.value = false
    }
  }

  async function createTunnel(localPort: number, remotePort: number, protocol: string) {
    const id = await tunnelService.create({ localPort, remotePort, protocol })
    await fetchTunnels()
    return id
  }

  async function deleteTunnel(id: string) {
    await tunnelService.delete(id)
    await fetchTunnels()
  }

  return {
    tunnels: computed(() => tunnels.value),
    loading: computed(() => loading.value),
    error: computed(() => error.value),
    createTunnel,
    deleteTunnel,
    fetchTunnels,
  }
}
