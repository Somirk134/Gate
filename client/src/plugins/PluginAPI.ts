import type { CommandRegistry } from '@/commands/CommandRegistry'
import type { AppContext } from '@/core/AppContext'
import type { EventBus } from '@/events/EventBus'
import type { ServiceRegistry } from '@/registry/ServiceRegistry'
import type { ConfigurationService } from '@/services/ConfigurationService'
import type { AppEventMap } from '@/types/application'

export interface PluginAPI {
  readonly app: AppContext
  readonly events: EventBus<AppEventMap>
  readonly commands: CommandRegistry
  readonly services: ServiceRegistry
  readonly configuration: ConfigurationService
}

export function createPluginAPI(app: AppContext): PluginAPI {
  return {
    get app() {
      return app
    },
    get events() {
      return app.events
    },
    get commands() {
      return app.commands
    },
    get services() {
      return app.services
    },
    get configuration() {
      return app.configuration
    },
  }
}
