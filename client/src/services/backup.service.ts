import { TauriIpcClient } from '@/ipc'
import { open, save } from '@tauri-apps/plugin-dialog'

const ipc = new TauriIpcClient()

export interface BackupContents {
  projects: number
  servers: number
  tunnels: number
  domains: number
  certificates: number
  settings: number
  runtimeConfig: number
}

export interface BackupSecurity {
  serverTokensIncluded: boolean
  certificatePrivateKeysIncluded: boolean
  certificatePemIncluded: boolean
  projectSecretsIncluded: boolean
  projectNotesIncluded: boolean
}

export interface BackupExportResult {
  path: string
  fileName: string
  size: number
  createdAt: string
  contents: BackupContents
  entries: number
}

export interface BackupPreview {
  valid: boolean
  path: string
  product: string
  version: string
  schemaVersion: number
  appVersion: string
  createdAt: string
  contents: BackupContents
  security: BackupSecurity
  notes: string[]
  entries: string[]
}

export interface BackupRestoreResult {
  ok: boolean
  restoredAt: number
  contents: BackupContents
  messageKey: string
}

const backupFilters = [
  {
    name: 'Gate Backup',
    extensions: ['gatebackup'],
  },
]

export const backupService = {
  async chooseExportPath(): Promise<string | null> {
    return save({
      defaultPath: 'gate-v0.9.gatebackup',
      filters: backupFilters,
    })
  },

  async chooseRestorePath(): Promise<string | null> {
    const selected = await open({
      multiple: false,
      filters: backupFilters,
    })
    if (!selected || Array.isArray(selected)) return null
    return selected
  },

  async export(destination: string): Promise<BackupExportResult> {
    return ipc.invoke<BackupExportResult>('backup_export', { destination })
  },

  async preview(path: string): Promise<BackupPreview> {
    return ipc.invoke<BackupPreview>('backup_preview', { path })
  },

  async restore(path: string): Promise<BackupRestoreResult> {
    return ipc.invoke<BackupRestoreResult>('backup_restore', { path })
  },
}
