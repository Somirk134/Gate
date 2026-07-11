import { onBeforeUnmount, onMounted, ref } from 'vue'
import { useDomainStore } from '../stores/domain'

const REFRESH_INTERVAL_MS = 8000

export function useDomains() {
  const store = useDomainStore()
  const timer = ref<number | null>(null)

  async function bootstrap() {
    await store.refresh()
    await store.loadList()
  }

  onMounted(async () => {
    await bootstrap()
    timer.value = window.setInterval(() => {
      void store.refresh()
    }, REFRESH_INTERVAL_MS)
  })

  onBeforeUnmount(() => {
    if (timer.value) {
      window.clearInterval(timer.value)
    }
  })

  return store
}

export function useAnimatedNumber(target: () => number, duration = 600) {
  const current = ref(0)
  let frame = 0

  function animate(next: number) {
    const start = current.value
    const delta = next - start
    const startedAt = performance.now()
    cancelAnimationFrame(frame)
    const tick = (now: number) => {
      const progress = Math.min(1, (now - startedAt) / duration)
      current.value = Math.round(start + delta * progress)
      if (progress < 1) {
        frame = requestAnimationFrame(tick)
      }
    }
    frame = requestAnimationFrame(tick)
  }

  return { current, animate, target }
}
