import type { App } from "vue"
import type { Router } from "vue-router"
import { EventBus } from "@/events/EventBus"
import type { AppEnvironment, AppEventMap } from "@/types/application"
import type { ConfigurationService } from "@/services/ConfigurationService"
import { CONFIGURATION_SERVICE } from "@/services/tokens"
import { registerBuiltinCommands } from "@/commands/registerBuiltinCommands"
import { AppRegistry } from "./AppRegistry"
import { ApplicationService } from "./ApplicationService"
import type { AppContext } from "./AppContext"
import { AppLifecycle } from "./lifecycle"
import { registerApplicationInitializers } from "./registerApplicationInitializers"
import { registerApplicationServices } from "./registerApplicationServices"

export interface AppBootstrapOptions {
  app: App
  router: Router
  environment?: Partial<AppEnvironment>
}

export class AppBootstrap {
  static async create(options: AppBootstrapOptions) {
    const registry = new AppRegistry()
    const events = new EventBus<AppEventMap>()
    const lifecycle = new AppLifecycle(events)
    const environment = AppBootstrap.createEnvironment(options.environment)

    const context: AppContext = {
      id: AppBootstrap.createAppId(),
      environment,
      startedAt: Date.now(),
      lifecycle,
      events,
      registry,
      initializers: registry.initializers,
      commands: registry.commands,
      services: registry.services,
      configuration: undefined as unknown as ConfigurationService,
    }

    registerApplicationServices(context)
    context.configuration = context.services.resolve(CONFIGURATION_SERVICE)
    context.commands.bindContext(context)
    registerBuiltinCommands(context, options.router)
    registerApplicationInitializers(context, options.app, options.router)

    const application = new ApplicationService(context)
    await application.initialize()

    return application
  }

  private static createEnvironment(environment: Partial<AppEnvironment> = {}): AppEnvironment {
    return {
      name: environment.name ?? "Gate",
      version: environment.version ?? "0.1.0",
      runtime: environment.runtime ?? "desktop",
      dev: environment.dev ?? Boolean((import.meta as unknown as { env?: { DEV?: boolean } }).env?.DEV),
    }
  }

  private static createAppId() {
    if (typeof crypto !== "undefined" && "randomUUID" in crypto) {
      return crypto.randomUUID()
    }

    return `app-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`
  }
}
