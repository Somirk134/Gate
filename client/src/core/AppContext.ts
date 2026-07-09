import type { CommandRegistry } from '@/commands/CommandRegistry'
import type { ConfigurationService } from '@/services/ConfigurationService'
import type { AppEnvironment, AppEventMap } from '@/types/application'
import type { EventBus } from '@/events/EventBus'
import type { ServiceRegistry } from '@/registry/ServiceRegistry'
import type { AppInitializerRegistry } from './AppInitializer'
import type { AppRegistry } from './AppRegistry'
import type { AppLifecycle } from './lifecycle'

export interface AppContext {
  id: string
  environment: AppEnvironment
  startedAt: number
  lifecycle: AppLifecycle
  events: EventBus<AppEventMap>
  registry: AppRegistry
  initializers: AppInitializerRegistry
  commands: CommandRegistry
  services: ServiceRegistry
  configuration: ConfigurationService
}
