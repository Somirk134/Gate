import type {
  EventEnvelope,
  EventHandler,
  EventName,
  EventPayload,
  SubscribeOptions,
  Unsubscribe,
} from './types'

interface Listener<TPayload = EventPayload> {
  id: number
  handler: EventHandler<TPayload>
  once: boolean
  priority: number
}

export class EventBus<TEvents extends object = Record<string, EventPayload>> {
  private readonly listeners = new Map<EventName, Listener[]>()
  private nextListenerId = 1

  subscribe<TKey extends keyof TEvents & string>(
    name: TKey,
    handler: EventHandler<TEvents[TKey]>,
    options: SubscribeOptions = {},
  ): Unsubscribe {
    const listener: Listener<TEvents[TKey]> = {
      id: this.nextListenerId,
      handler,
      once: Boolean(options.once),
      priority: options.priority ?? 0,
    }

    this.nextListenerId += 1

    const listeners = this.listeners.get(name) ?? []
    listeners.push(listener as Listener)
    listeners.sort((a, b) => b.priority - a.priority)
    this.listeners.set(name, listeners)

    const unsubscribe = () => this.unsubscribe(name, listener.id)

    options.signal?.addEventListener('abort', unsubscribe, { once: true })

    return unsubscribe
  }

  once<TKey extends keyof TEvents & string>(
    name: TKey,
    handler: EventHandler<TEvents[TKey]>,
    priority = 0,
  ): Unsubscribe {
    return this.subscribe(name, handler, { once: true, priority })
  }

  unsubscribe(name: EventName, listenerId: number) {
    const listeners = this.listeners.get(name)

    if (!listeners) {
      return
    }

    const next = listeners.filter((listener) => listener.id !== listenerId)

    if (next.length === 0) {
      this.listeners.delete(name)
      return
    }

    this.listeners.set(name, next)
  }

  async publish<TKey extends keyof TEvents & string>(
    name: TKey,
    payload: TEvents[TKey],
    source?: string,
  ) {
    const envelope: EventEnvelope<TEvents[TKey]> = {
      name,
      payload,
      source,
      timestamp: Date.now(),
    }

    const listeners = [...(this.listeners.get(name) ?? [])]

    for (const listener of listeners) {
      await listener.handler(envelope)

      if (listener.once) {
        this.unsubscribe(name, listener.id)
      }
    }
  }

  clear(name?: EventName) {
    if (name) {
      this.listeners.delete(name)
      return
    }

    this.listeners.clear()
  }

  listenerCount(name?: EventName) {
    if (name) {
      return this.listeners.get(name)?.length ?? 0
    }

    return Array.from(this.listeners.values()).reduce((total, listeners) => {
      return total + listeners.length
    }, 0)
  }
}
