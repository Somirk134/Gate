import { onBeforeUnmount, onMounted, ref } from 'vue'
import type { useTunnelStore } from '../store/tunnel'

export function useTunnelMonitor(store: ReturnType<typeof useTunnelStore>, intervalMs = 5000) {
  const active = ref(false)
  let timer: ReturnType<typeof setInterval> | null = null

  function start() {
    if (active.value) return
    active.value = true
    timer = setInterval(() => {
      if (store.status !== 'loading') {
        void store.refresh()
      }
    }, intervalMs)
  }

  function stop() {
    active.value = false
    if (timer) {
      clearInterval(timer)
      timer = null
    }
  }

  function setInterval_(ms: number) {
    const wasActive = active.value
    stop()
    intervalMs = ms
    if (wasActive) start()
  }

  onMounted(start)
  onBeforeUnmount(stop)

  return { active, start, stop, setInterval: setInterval_ }
}
