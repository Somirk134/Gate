import type { PluginLifecycle, PluginLifecycleState } from "./lifecycle/PluginLifecycle"

export interface PluginManifest {
  id: string
  name: string
  version: string
  description?: string
  contributes?: {
    commands?: string[]
    views?: string[]
    shortcuts?: string[]
  }
}

export interface RegisteredPlugin {
  manifest: PluginManifest
  lifecycle: PluginLifecycle
  state: PluginLifecycleState
}

export class PluginRegistry {
  private readonly plugins = new Map<string, RegisteredPlugin>()

  register(manifest: PluginManifest, lifecycle: PluginLifecycle) {
    if (this.plugins.has(manifest.id)) {
      throw new Error(`Plugin already registered: ${manifest.id}`)
    }

    this.plugins.set(manifest.id, {
      manifest,
      lifecycle,
      state: "registered",
    })
  }

  unregister(id: string) {
    this.plugins.delete(id)
  }

  get(id: string) {
    return this.plugins.get(id)
  }

  list() {
    return Array.from(this.plugins.values())
  }

  setState(id: string, state: PluginLifecycleState) {
    const plugin = this.plugins.get(id)

    if (plugin) {
      plugin.state = state
    }
  }
}
