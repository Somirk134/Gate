import type { AppContext } from './AppContext'
import { AppLifecyclePhase } from './lifecycle'

export class ApplicationService {
  constructor(readonly context: AppContext) {}

  async initialize() {
    await this.context.lifecycle.transitionTo(
      AppLifecyclePhase.Initializing,
      'application bootstrap',
    )
    await this.context.initializers.run(this.context)
    await this.context.lifecycle.transitionTo(AppLifecyclePhase.Ready, 'initializers complete')
    await this.context.events.publish('app:ready', { at: Date.now() })
  }

  async run() {
    await this.context.lifecycle.transitionTo(AppLifecyclePhase.Running, 'vue mounted')
  }

  async update() {
    await this.context.lifecycle.transitionTo(AppLifecyclePhase.Updating, 'update requested')
  }

  async restart() {
    await this.context.lifecycle.transitionTo(AppLifecyclePhase.Restarting, 'restart requested')
  }

  async close() {
    await this.context.lifecycle.transitionTo(AppLifecyclePhase.Closing, 'application closing')
    await this.context.registry.dispose()
    await this.context.lifecycle.transitionTo(AppLifecyclePhase.Exit, 'application disposed')
  }
}
