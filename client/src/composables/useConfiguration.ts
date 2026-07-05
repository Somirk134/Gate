import { onUnmounted, ref } from "vue"
import { useAppContext } from "@/providers/appContext"
import type { AppConfiguration } from "@/services/ConfigurationService"

export function useConfiguration() {
  const context = useAppContext()
  const snapshot = ref<AppConfiguration>(context.configuration.snapshot())

  const subscription = context.configuration.watch("*", () => {
    snapshot.value = context.configuration.snapshot()
  })
  onUnmounted(() => {
    void subscription.dispose()
  })

  function get<T = unknown>(key: string) {
    return context.configuration.get<T>(key)
  }

  function set<T = unknown>(key: string, value: T) {
    context.configuration.set(key, value)
  }

  function reset(key?: string) {
    context.configuration.reset(key)
  }

  return {
    snapshot,
    get,
    set,
    reset,
    dispose: subscription.dispose,
  }
}
