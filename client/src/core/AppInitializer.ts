import type { AppContext } from './AppContext'
import { GateAppError } from '@/ipc'

export interface AppInitializer {
  id: string
  priority?: number
  initialize(context: AppContext): void | Promise<void>
}

export class AppInitializerRegistry {
  private readonly initializers = new Map<string, AppInitializer>()

  register(initializer: AppInitializer) {
    if (this.initializers.has(initializer.id)) {
      throw new GateAppError({
        code: 'APP_INITIALIZER_ALREADY_REGISTERED',
        messageKey: 'errors.application.initializerAlreadyRegistered',
        details: { id: initializer.id },
        timestamp: Date.now(),
      })
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
