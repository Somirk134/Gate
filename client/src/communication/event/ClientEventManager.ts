import { createCommunicationId } from '../shared/id'
import type {
  Event as CommunicationEvent,
  EventHandler,
  EventName,
  SubscribeOptions,
  Unsubscribe,
} from '../types'

interface Listener<TPayload = unknown> {
  id: string
  handler: EventHandler<TPayload>
  once: boolean
  priority: number
}

export class ClientEventManager {
  private readonly listeners = new Map<EventName, Listener[]>()

  subscribe<TPayload>(
    name: EventName,
    handler: EventHandler<TPayload>,
    options: SubscribeOptions = {},
  ): Unsubscribe {
    const listener: Listener<TPayload> = {
      id: createCommunicationId('sub'),
      handler,
      once: Boolean(options.once),
      priority: options.priority ?? 0,
    }

    const listeners = this.listeners.get(name) ?? []
    listeners.push(listener as Listener)
    listeners.sort((left, right) => right.priority - left.priority)
    this.listeners.set(name, listeners)

    const unsubscribe = () => this.unsubscribe(name, listener.id)
    options.signal?.addEventListener('abort', unsubscribe, { once: true })

    return unsubscribe
  }

  unsubscribe(name: EventName, listenerId: string) {
    const listeners = this.listeners.get(name)

    if (!listeners) {
      return false
    }

    const next = listeners.filter((listener) => listener.id !== listenerId)

    if (next.length === 0) {
      this.listeners.delete(name)
    } else {
      this.listeners.set(name, next)
    }

    return next.length !== listeners.length
  }

  async publish<TPayload>(event: CommunicationEvent<TPayload>) {
    const listeners = [...(this.listeners.get(event.name) ?? [])]

    for (const listener of listeners) {
      await listener.handler(event)

      if (listener.once) {
        this.unsubscribe(event.name, listener.id)
      }
    }
  }

  async broadcast(events: CommunicationEvent[]) {
    for (const event of events) {
      await this.publish(event)
    }
  }

  clear(name?: EventName) {
    if (name) {
      this.listeners.delete(name)
      return
    }

    this.listeners.clear()
  }

  subscriberCount(name?: EventName) {
    if (name) {
      return this.listeners.get(name)?.length ?? 0
    }

    return Array.from(this.listeners.values()).reduce((total, listeners) => {
      return total + listeners.length
    }, 0)
  }
}
