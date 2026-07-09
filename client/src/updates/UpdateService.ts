import type { EventBus } from '@/events/EventBus'
import type { AppEventMap } from '@/types/application'
import { GITHUB_REPOSITORY_URL } from '@/constants'
import { GateAppError } from '@/ipc'
import { isTauri } from '@tauri-apps/api/core'
import { relaunch } from '@tauri-apps/plugin-process'
import { check, type Update } from '@tauri-apps/plugin-updater'

export type UpdateStatus =
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
  source?: 'tauri' | 'github'
}

export interface AutoUpdateService {
  getStatus(): UpdateStatus
  check(): Promise<UpdateInfo>
  download(): Promise<void>
  install(): Promise<void>
  restart(): Promise<void>
}

interface GitHubReleaseResponse {
  tag_name?: string
  name?: string
  body?: string
  html_url?: string
  published_at?: string
  draft?: boolean
  prerelease?: boolean
}

const GITHUB_RELEASES_API = 'https://api.github.com/repos/Somirk134/Gate/releases?per_page=10'

export class TauriAutoUpdateService implements AutoUpdateService {
  private status: UpdateStatus = 'idle'
  // 缓存已检查到的 Tauri 更新资源，后续下载和安装复用同一个真实更新对象。
  private pendingUpdate: Update | null = null

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
      const tauriUpdate = await this.checkWithTauriUpdater()

      if (tauriUpdate) {
        this.pendingUpdate = tauriUpdate
        this.setStatus('available')
        return {
          available: true,
          currentVersion: tauriUpdate.currentVersion,
          version: tauriUpdate.version,
          notes: tauriUpdate.body,
          date: tauriUpdate.date,
          installable: true,
          source: 'tauri',
        }
      }

      const githubUpdate = await this.checkWithGitHubReleases()
      this.setStatus(githubUpdate.available ? 'available' : 'idle')
      return githubUpdate
    } catch (error) {
      this.setStatus('error')
      throw error
    }
  }

  async download() {
    const update = this.pendingUpdate

    if (!update) {
      throw new GateAppError({
        code: 'UPDATE_DOWNLOAD_UNAVAILABLE',
        messageKey: 'errors.updateNoDownload',
        timestamp: Date.now(),
      })
    }

    this.setStatus('downloading')

    try {
      await update.download()
      this.setStatus('ready')
    } catch (error) {
      this.setStatus('error')
      throw error
    }
  }

  async install() {
    const update = this.pendingUpdate

    if (!update) {
      throw new GateAppError({
        code: 'UPDATE_INSTALL_UNAVAILABLE',
        messageKey: 'errors.updateNoInstall',
        timestamp: Date.now(),
      })
    }

    this.setStatus('installing')

    try {
      await update.install()
      this.setStatus('installed')
    } catch (error) {
      this.setStatus('error')
      throw error
    }
  }

  async restart() {
    await relaunch()
  }

  private async checkWithTauriUpdater() {
    if (!isTauri()) {
      return null
    }

    try {
      return await check({ timeout: 15_000 })
    } catch {
      // Tauri 更新服务端未部署或网络异常时，降级到 GitHub Release 检查，保证按钮仍能真实判断版本。
      return null
    }
  }

  private async checkWithGitHubReleases(): Promise<UpdateInfo> {
    const response = await fetch(GITHUB_RELEASES_API, {
      headers: {
        Accept: 'application/vnd.github+json',
      },
    })

    if (response.status === 404) {
      await this.closePendingUpdate()
      return {
        available: false,
        currentVersion: this.currentVersion,
        installable: false,
        source: 'github',
      }
    }

    if (!response.ok) {
      throw new GateAppError({
        code: 'UPDATE_CHECK_HTTP_FAILED',
        messageKey: 'errors.updateCheckHttpFailed',
        details: { status: response.status },
        timestamp: Date.now(),
      })
    }

    const releases = (await response.json()) as GitHubReleaseResponse[]
    const release = releases.find((item) => !item.draft)
    if (!release) {
      await this.closePendingUpdate()
      return {
        available: false,
        currentVersion: this.currentVersion,
        installable: false,
        source: 'github',
      }
    }

    const latestVersion = release.tag_name ?? release.name
    const available = Boolean(
      latestVersion && isNewerVersion(latestVersion, this.currentVersion),
    )

    if (!available) {
      await this.closePendingUpdate()
    }

    return {
      available,
      currentVersion: this.currentVersion,
      version: latestVersion,
      notes: release.body,
      date: release.published_at,
      url: release.html_url ?? `${GITHUB_REPOSITORY_URL}/releases`,
      installable: false,
      source: 'github',
    }
  }

  private async closePendingUpdate() {
    if (!this.pendingUpdate) return

    await this.pendingUpdate.close().catch(() => undefined)
    this.pendingUpdate = null
  }

  private setStatus(status: UpdateStatus) {
    this.status = status
    void this.events.publish('update:status', { status })
  }
}

function isNewerVersion(candidate: string, current: string) {
  const candidateParts = normalizeVersion(candidate)
  const currentParts = normalizeVersion(current)
  const length = Math.max(candidateParts.length, currentParts.length)

  for (let index = 0; index < length; index += 1) {
    const candidatePart = candidateParts[index] ?? 0
    const currentPart = currentParts[index] ?? 0

    if (candidatePart > currentPart) return true
    if (candidatePart < currentPart) return false
  }

  return false
}

function normalizeVersion(version: string) {
  return version
    .trim()
    .replace(/^v/i, '')
    .split(/[+-]/)[0]
    .split('.')
    .map((part) => Number.parseInt(part, 10))
    .map((part) => (Number.isFinite(part) ? part : 0))
}
