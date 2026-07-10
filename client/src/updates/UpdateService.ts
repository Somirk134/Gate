import type { EventBus } from '@/events/EventBus'
import type { AppEventMap } from '@/types/application'
import { APP_RELEASE_CHANNEL, GITHUB_REPOSITORY_URL } from '@/constants'
import { GateAppError } from '@/ipc'
import { relaunch } from '@tauri-apps/plugin-process'

export type UpdateStatus =
  | 'disabled'
  | 'idle'
  | 'checking'
  | 'available'
  | 'downloading'
  | 'ready'
  | 'installing'
  | 'installed'
  | 'error'

export interface UpdateInfo {
  available: boolean
  currentVersion?: string
  version?: string
  notes?: string
  date?: string
  url?: string
  installable?: boolean
  source?: 'disabled' | 'github'
  channel?: string
}

export interface AutoUpdateService {
  getStatus(): UpdateStatus
  check(): Promise<UpdateInfo>
  download(): Promise<void>
  install(): Promise<void>
  restart(): Promise<void>
}

export class TauriAutoUpdateService implements AutoUpdateService {
  private status: UpdateStatus = 'disabled'

  constructor(
    private readonly events: EventBus<AppEventMap>,
    private readonly currentVersion: string,
  ) {}

  getStatus() {
    return this.status
  }

  async check(): Promise<UpdateInfo> {
    this.setStatus('disabled')
    return {
      available: false,
      currentVersion: this.currentVersion,
      version: this.currentVersion,
      url: `${GITHUB_REPOSITORY_URL}/releases`,
      installable: false,
      source: 'disabled',
      channel: APP_RELEASE_CHANNEL,
    }
  }

  async download() {
    throw new GateAppError({
      code: 'UPDATE_DOWNLOAD_UNAVAILABLE',
      messageKey: 'errors.updateNoDownload',
      timestamp: Date.now(),
    })
  }

  async install() {
    throw new GateAppError({
      code: 'UPDATE_INSTALL_UNAVAILABLE',
      messageKey: 'errors.updateNoInstall',
      timestamp: Date.now(),
    })
  }

  async restart() {
    await relaunch()
  }

  private setStatus(status: UpdateStatus) {
    this.status = status
    void this.events.publish('update:status', { status })
  }
}
