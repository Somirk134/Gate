import type { EventBus } from "@/events/EventBus"
import type { AppEventMap, NotificationPayload } from "@/types/application"

export interface NotificationService {
  show(notification: NotificationPayload): string
  success(title: string, content?: string): string
  error(title: string, content?: string): string
  warning(title: string, content?: string): string
  info(title: string, content?: string): string
}

export class EventNotificationService implements NotificationService {
  constructor(private readonly events: EventBus<AppEventMap>) {}

  show(notification: NotificationPayload) {
    const id = notification.id ?? `notification-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`

    void this.events.publish("notification:show", {
      ...notification,
      id,
    })

    return id
  }

  success(title: string, content?: string) {
    return this.show({ type: "success", title, content })
  }

  error(title: string, content?: string) {
    return this.show({ type: "error", title, content })
  }

  warning(title: string, content?: string) {
    return this.show({ type: "warning", title, content })
  }

  info(title: string, content?: string) {
    return this.show({ type: "info", title, content })
  }
}
