import { CommandRegistry } from "@/commands/CommandRegistry"
import { ServiceRegistry } from "@/registry/ServiceRegistry"
import { AppInitializerRegistry, type AppInitializer } from "./AppInitializer"

export class AppRegistry {
  readonly services = new ServiceRegistry()
  readonly commands = new CommandRegistry()
  readonly initializers = new AppInitializerRegistry()

  registerInitializer(initializer: AppInitializer) {
    this.initializers.register(initializer)
  }

  async dispose() {
    this.commands.dispose()
    this.initializers.clear()
    await this.services.dispose()
  }
}
