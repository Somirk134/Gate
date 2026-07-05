import { EventBus } from "@/events/EventBus"
import { MemoryStorageService } from "@/storage"
import { DefaultConfigurationService } from "@/services/ConfigurationService"
import { DefaultLoggerService } from "@/logger/LoggerService"
import { DefaultThemeService, type ThemeMode } from "@/theme/ThemeService"
import type { AppContext } from "@/core/AppContext"
import { AppLifecycle, AppLifecyclePhase } from "@/core/lifecycle"
import { AppRegistry } from "@/core/AppRegistry"
import type { AppEventMap } from "@/types/application"
import {
  CONFIGURATION_SERVICE,
  LOGGER_SERVICE,
  STORAGE_SERVICE,
  THEME_SERVICE,
} from "@/services/tokens"

export class ApplicationMock {
  readonly context: AppContext

  constructor() {
    const events = new EventBus<AppEventMap>()
    const lifecycle = new AppLifecycle(events)
    const registry = new AppRegistry()
    const services = registry.services
    const commands = registry.commands
    const initializers = registry.initializers
    const storage = new MemoryStorageService()
    const configuration = new DefaultConfigurationService(storage, events)

    services.registerInstance(STORAGE_SERVICE, storage)
    services.registerInstance(CONFIGURATION_SERVICE, configuration)
    services.registerInstance(LOGGER_SERVICE, new DefaultLoggerService())
    services.register(
      THEME_SERVICE,
      () => new DefaultThemeService(configuration, events),
      { eager: true },
    )

    this.context = {
      id: "mock-app",
      environment: {
        name: "Gate Mock",
        version: "0.0.0",
        runtime: "test",
        dev: true,
      },
      startedAt: Date.now(),
      lifecycle,
      events,
      registry,
      initializers,
      commands,
      services,
      configuration,
    }

    commands.bindContext(this.context)
  }

  async start() {
    await this.context.lifecycle.transitionTo(AppLifecyclePhase.Initializing, "mock start")
    await this.context.lifecycle.transitionTo(AppLifecyclePhase.Ready, "mock ready")
    await this.context.events.publish("app:ready", { at: Date.now() })
    await this.context.lifecycle.transitionTo(AppLifecyclePhase.Running, "mock running")
  }

  async close() {
    await this.context.lifecycle.transitionTo(AppLifecyclePhase.Closing, "mock close")
    await this.context.lifecycle.transitionTo(AppLifecyclePhase.Exit, "mock exit")
  }

  switchTheme(mode: ThemeMode) {
    this.context.services.resolve(THEME_SERVICE).setTheme(mode)
  }

  publish<TKey extends keyof AppEventMap & string>(
    name: TKey,
    payload: AppEventMap[TKey],
  ) {
    return this.context.events.publish(name, payload, "application-mock")
  }
}

export function createApplicationMock() {
  return new ApplicationMock()
}
