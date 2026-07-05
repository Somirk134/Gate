import { onUnmounted, ref } from "vue"
import { useAppContext } from "@/providers/appContext"

export function useApplication() {
  const app = useAppContext()
  const phase = ref(app.lifecycle.phase)
  const unsubscribe = app.events.subscribe("lifecycle:transition", ({ payload }) => {
    phase.value = payload.to
  })

  onUnmounted(unsubscribe)

  return {
    app,
    lifecycle: app.lifecycle,
    phase,
    environment: app.environment,
    services: app.services,
    commands: app.commands,
    events: app.events,
    configuration: app.configuration,
  }
}
