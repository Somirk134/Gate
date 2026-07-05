import type { App } from "vue"
import type { EventBus } from "@/events/EventBus"
import type { LoggerService } from "@/logger/LoggerService"
import type { NotificationService } from "@/services/NotificationService"
import type { AppEventMap } from "@/types/application"
import type { Disposable } from "@/utils/disposable"

export interface ErrorHandler extends Disposable {
  capture(error: unknown, context?: string, fatal?: boolean): void
  installVue(app: App): void
  start(): void
}

export class GlobalErrorHandler implements ErrorHandler {
  private started = false

  private readonly handleError = (event: ErrorEvent) => {
    this.capture(event.error ?? event.message, "window.error")
  }

  private readonly handleRejection = (event: PromiseRejectionEvent) => {
    this.capture(event.reason, "window.unhandledrejection")
  }

  constructor(
    private readonly logger: LoggerService,
    private readonly events: EventBus<AppEventMap>,
    private readonly notifications: NotificationService,
  ) {}

  start() {
    if (this.started || typeof window === "undefined") {
      return
    }

    window.addEventListener("error", this.handleError)
    window.addEventListener("unhandledrejection", this.handleRejection)
    this.started = true
  }

  installVue(app: App) {
    app.config.errorHandler = (error, _instance, info) => {
      this.capture(error, `vue:${info}`)
    }
  }

  capture(error: unknown, context = "application", fatal = false) {
    const message = this.toMessage(error)
    this.logger.error(message, { error, context, fatal })
    this.notifications.error("Application Error", message)
    void this.events.publish("app:error", {
      error,
      message,
      context,
      fatal,
    })
  }

  dispose() {
    if (!this.started || typeof window === "undefined") {
      return
    }

    window.removeEventListener("error", this.handleError)
    window.removeEventListener("unhandledrejection", this.handleRejection)
    this.started = false
  }

  private toMessage(error: unknown) {
    if (error instanceof Error) {
      return error.message
    }

    if (typeof error === "string") {
      return error
    }

    return "Unknown application error"
  }
}
