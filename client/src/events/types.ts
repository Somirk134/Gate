export type EventName = string
export type EventPayload = unknown

export interface EventEnvelope<TPayload = EventPayload> {
  name: EventName
  payload: TPayload
  timestamp: number
  source?: string
}

export type EventHandler<TPayload = EventPayload> = (
  event: EventEnvelope<TPayload>,
) => void | Promise<void>

export interface SubscribeOptions {
  once?: boolean
  priority?: number
  signal?: AbortSignal
}

export type Unsubscribe = () => void
