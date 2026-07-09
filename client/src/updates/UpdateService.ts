import type { EventBus } from '@/events/EventBus'
import type { AppEventMap } from '@/types/application'

export type UpdateStatus = 'idle' | 'checking' | 'available' | 'downloading' | 'ready' | 'error'

export interface UpdateInfo {
  available: boolean
  version?: string
  notes?: string
}

export interface AutoUpdateService {
  getStatus(): UpdateStatus
  check(): Promise<UpdateInfo>
  download(): Promise<void>
  install(): Promise<void>
  restart(): Promise<void>
}

export class StubAutoUpdateService implements AutoUpdateService {
  private status: UpdateStatus = 'idle'

  constructor(private readonly events: EventBus<AppEventMap>) {}

  getStatus() {
    return this.status
  }

  async check(): Promise<UpdateInfo> {
    this.setStatus('checking')
    this.setStatus('idle')
    return { available: false }
  }

  async download() {
    this.setStatus('idle')
  }

  async install() {
    this.setStatus('idle')
  }

  async restart() {
    this.setStatus('idle')
  }

  private setStatus(status: UpdateStatus) {
    this.status = status
    void this.events.publish('update:status', { status })
  }
}
