import { onUnmounted } from 'vue'
import { useAppContext } from '@/providers/appContext'
import type { EventHandler, SubscribeOptions } from '@/events/types'
import type { AppEventMap } from '@/types/application'

export function useEventBus() {
  const context = useAppContext()

  function publish<TKey extends keyof AppEventMap & string>(
    name: TKey,
    payload: AppEventMap[TKey],
    source?: string,
  ) {
    return context.events.publish(name, payload, source)
  }

  function subscribe<TKey extends keyof AppEventMap & string>(
    name: TKey,
    handler: EventHandler<AppEventMap[TKey]>,
    options?: SubscribeOptions,
  ) {
    const unsubscribe = context.events.subscribe(name, handler, options)
    onUnmounted(unsubscribe)
    return unsubscribe
  }

  return {
    events: context.events,
    publish,
    subscribe,
  }
}
