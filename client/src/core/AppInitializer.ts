import type { AppContext } from "./AppContext"

export interface AppInitializer {
  id: string
  priority?: number
  initialize(context: AppContext): void | Promise<void>
}

export class AppInitializerRegistry {
  private readonly initializers = new Map<string, AppInitializer>()

  register(initializer: AppInitializer) {
    if (this.initializers.has(initializer.id)) {
      throw new Error(`App initializer already registered: ${initializer.id}`)
    }

    this.initializers.set(initializer.id, initializer)
  }

  list() {
    return Array.from(this.initializers.values()).sort((a, b) => {
      return (b.priority ?? 0) - (a.priority ?? 0)
    })
  }

  async run(context: AppContext) {
    for (const initializer of this.list()) {
      await initializer.initialize(context)
    }
  }

  clear() {
    this.initializers.clear()
  }
}
