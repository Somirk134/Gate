import { invoke } from '@tauri-apps/api/core'
import type { EventBus } from '@/events/EventBus'
import type { AppEventMap } from '@/types/application'
import { APP_RELEASE_CHANNEL, GITHUB_REPOSITORY_URL } from '@/constants'
import { relaunch } from '@tauri-apps/plugin-process'

/** 从 GitHub 仓库 owner/repo 解析出 API 路径。 */
function githubReleasesApiUrl(repoUrl: string): string {
  try {
    const u = new URL(repoUrl)
    // https://github.com/Somirk134/Gate → /repos/Somirk134/Gate/releases/latest
    return `https://api.github.com/repos${u.pathname}/releases/latest`
  } catch {
    return 'https://api.github.com/repos/Somirk134/Gate/releases/latest'
  }
}

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
  source?: 'github' | 'github-api'
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

/** GitHub Releases API 返回的最新 Release 结构。 */
interface GitHubRelease {
  tag_name: string
  name: string | null
  body: string | null
  published_at: string | null
  html_url: string
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
      this.setStatus(raw.available && raw.installable ? 'available' : 'idle')
      return {
        available: raw.available,
        currentVersion: raw.currentVersion || this.currentVersion,
        version: raw.version ?? undefined,
        notes: raw.notes ?? undefined,
        date: raw.date ?? undefined,
        url: raw.url,
        installable: raw.installable,
        source: raw.available ? 'github' : undefined,
        channel: APP_RELEASE_CHANNEL,
      }
    } catch {
      // 后端命令不可用时，回退到 GitHub Releases API 直接查询。
      return this.checkViaGithubApi()
    }
  }

  /** 通过公开的 GitHub REST API 检查最新 Release 版本。 */
  private async checkViaGithubApi(): Promise<UpdateInfo> {
    try {
      const apiUrl = githubReleasesApiUrl(GITHUB_REPOSITORY_URL)
      const res = await fetch(apiUrl, { headers: { Accept: 'application/vnd.github.v3+json' } })
      if (!res.ok) throw new Error(`HTTP ${res.status}`)

      const release: GitHubRelease = await res.json()
      const latestVersion = release.tag_name.replace(/^v/i, '')
      const releasesUrl = `${GITHUB_REPOSITORY_URL}/releases`

      // 简单语义版本比较：去掉前缀 v 后逐段比较。
      const isNewer = this.compareVersions(latestVersion, this.currentVersion) > 0

      if (isNewer) {
        this.setStatus('available')
        return {
          available: true,
          currentVersion: this.currentVersion,
          version: latestVersion,
          notes: release.body ?? undefined,
          date: release.published_at ?? undefined,
          url: releasesUrl,
          installable: false, // 通过 API 查询无法获取签名安装包，引导用户手动更新
          source: 'github-api',
          channel: APP_RELEASE_CHANNEL,
        }
      }

      this.setStatus('idle')
      return {
        available: false,
        currentVersion: this.currentVersion,
        version: latestVersion,
        url: releasesUrl,
        installable: false,
        source: 'github-api',
        channel: APP_RELEASE_CHANNEL,
      }
    } catch (err) {
      this.setStatus('error')
      return {
        available: false,
        currentVersion: this.currentVersion,
        url: `${GITHUB_REPOSITORY_URL}/releases`,
        installable: false,
      }
    }
  }

  /**
   * 简易语义版本比较。
   * 返回正数表示 a > b（a 更新），0 表示相等，负数表示 a < b。
   */
  private compareVersions(a: string, b: string): number {
    const pa = a.split('.').map(Number)
    const pb = b.split('.').map(Number)
    const len = Math.max(pa.length, pb.length)
    for (let i = 0; i < len; i++) {
      const va = pa[i] ?? 0
      const vb = pb[i] ?? 0
      if (va !== vb) return va - vb
    }
    return 0
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
