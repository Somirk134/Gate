import { invoke } from '@tauri-apps/api/core'
import type { EventBus } from '@/events/EventBus'
import type { AppEventMap } from '@/types/application'
import { APP_RELEASE_CHANNEL, GITHUB_REPOSITORY_URL } from '@/constants'
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

// 后端 check_for_updates 命令返回的载荷结构（camelCase）。
interface RawUpdateInfo {
  available: boolean
  currentVersion: string
  version: string | null
  notes: string | null
  date: string | null
  url: string
  installable: boolean
}

export class TauriAutoUpdateService implements AutoUpdateService {
  private status: UpdateStatus = 'idle'

  constructor(
    private readonly events: EventBus<AppEventMap>,
    private readonly currentVersion: string,
  ) {}

  getStatus() {
    return this.status
  }

  async check(): Promise<UpdateInfo> {
    this.setStatus('checking')
    try {
      const raw = await invoke<RawUpdateInfo>('check_for_updates', {
        channel: APP_RELEASE_CHANNEL,
      })
      // 有可用更新且可应用内安装时进入 available，否则视为已是最新。
      this.setStatus(raw.available && raw.installable ? 'available' : 'idle')
      return {
        available: raw.available,
        currentVersion: raw.currentVersion || this.currentVersion,
        version: raw.version ?? undefined,
        notes: raw.notes ?? undefined,
        date: raw.date ?? undefined,
        url: raw.url,
        installable: raw.installable,
        source: raw.available ? 'github' : 'disabled',
        channel: APP_RELEASE_CHANNEL,
      }
    } catch {
      // 后端未注册命令或 updater 未配置时降级为禁用分支。
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
  }

  async download() {
    this.setStatus('downloading')
    try {
      await invoke('download_update')
      this.setStatus('ready')
    } catch (error) {
      this.setStatus('error')
      throw error
    }
  }

  async install() {
    this.setStatus('installing')
    try {
      await invoke('install_update')
      this.setStatus('installed')
    } catch (error) {
      this.setStatus('error')
      throw error
    }
  }

  async restart() {
    await relaunch()
  }

  private setStatus(status: UpdateStatus) {
    this.status = status
    void this.events.publish('update:status', { status })
  }
}
