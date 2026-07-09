import { EventBus } from '@/events/EventBus'
import type { AppEventMap } from '@/types/application'

export enum AppLifecyclePhase {
  Starting = 'starting',
  Initializing = 'initializing',
  Ready = 'ready',
  Running = 'running',
  Updating = 'updating',
  Restarting = 'restarting',
  Closing = 'closing',
  Exit = 'exit',
}

export interface AppLifecycleTransition {
  from: AppLifecyclePhase | null
  to: AppLifecyclePhase
  at: number
  reason?: string
}

export class AppLifecycle {
  private currentPhase: AppLifecyclePhase = AppLifecyclePhase.Starting
  private readonly transitions: AppLifecycleTransition[] = []

  constructor(private readonly events: EventBus<AppEventMap>) {}

  get phase() {
    return this.currentPhase
  }

  get history() {
    return [...this.transitions]
  }

  is(phase: AppLifecyclePhase) {
    return this.currentPhase === phase
  }

  async transitionTo(phase: AppLifecyclePhase, reason?: string) {
    if (phase === this.currentPhase) {
      return
    }

    const transition: AppLifecycleTransition = {
      from: this.currentPhase,
      to: phase,
      at: Date.now(),
      reason,
    }

    this.currentPhase = phase
    this.transitions.push(transition)
    await this.events.publish('lifecycle:transition', transition, 'app-lifecycle')
  }
}
