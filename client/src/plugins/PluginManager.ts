import type { EventBus } from "@/events/EventBus"
import type { AppEventMap } from "@/types/application"
import type { PluginAPI } from "./PluginAPI"
import { PluginRegistry, type PluginManifest } from "./PluginRegistry"
import type { PluginLifecycle } from "./lifecycle/PluginLifecycle"

export interface PluginManager {
  readonly registry: PluginRegistry
  register(manifest: PluginManifest, lifecycle: PluginLifecycle): void
  activate(id: string): Promise<void>
  deactivate(id: string): Promise<void>
  list(): ReturnType<PluginRegistry["list"]>
}

export class DefaultPluginManager implements PluginManager {
  readonly registry = new PluginRegistry()

  constructor(
    private readonly api: PluginAPI,
    private readonly events: EventBus<AppEventMap>,
  ) {}

  register(manifest: PluginManifest, lifecycle: PluginLifecycle) {
    this.registry.register(manifest, lifecycle)
  }

  async activate(id: string) {
    const plugin = this.requirePlugin(id)

    if (plugin.state === "active") {
      return
    }

    try {
      this.registry.setState(id, "activating")
      await plugin.lifecycle.activate(this.api)
      this.registry.setState(id, "active")
      await this.events.publish("plugin:activated", { id })
    } catch (error) {
      this.registry.setState(id, "failed")
      throw error
    }
  }

  async deactivate(id: string) {
    const plugin = this.requirePlugin(id)

    if (plugin.state !== "active") {
      return
    }

    this.registry.setState(id, "deactivating")
    await plugin.lifecycle.deactivate?.(this.api)
    this.registry.setState(id, "inactive")
    await this.events.publish("plugin:deactivated", { id })
  }

  list() {
    return this.registry.list()
  }

  private requirePlugin(id: string) {
    const plugin = this.registry.get(id)

    if (!plugin) {
      throw new Error(`Plugin not registered: ${id}`)
    }

    return plugin
  }
}
