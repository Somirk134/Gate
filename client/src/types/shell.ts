/* ==================================================================
   Shell 类型系统 — 纯框架类型，无业务
   ================================================================== */

// ── Navigation ──
export interface NavItem {
  name: string
  path: string
  label: string
  icon: string
  shortcut?: string
  children?: NavItem[]
  meta?: Record<string, any>
}

// ── Breadcrumb ──
export interface BreadcrumbItem {
  label: string
  path?: string
  icon?: string
}

// ── Command Palette ──
export interface CommandItem {
  id: string
  title: string
  subtitle?: string
  icon?: string
  shortcut?: string
  category: CommandCategory
  action: () => void | Promise<void>
  keywords?: string[]
}

export type CommandCategory = string

// ── Inspector ──
export interface InspectorTab {
  id: string
  label: string
  icon?: string
  component?: string
  slot?: string
}

// ── Dialog ──
export type DialogType = 'modal' | 'confirm' | 'alert' | 'delete' | 'form'

export interface DialogOptions {
  type: DialogType
  title: string
  content?: string
  props?: Record<string, any>
}

// ── Notification ──
export type NotificationType = 'success' | 'error' | 'warning' | 'info'

export interface NotificationOptions {
  type: NotificationType
  title: string
  content?: string
  duration?: number
  closable?: boolean
}

// ── Loading ──
export interface LoadingTask {
  id: string
  message: string
  progress?: number
  indeterminate?: boolean
}

// ── Status Bar ──
export interface StatusBarItem {
  id: string
  label: string
  value?: string
  icon?: string
  status?: 'online' | 'offline' | 'warning' | 'error' | 'info'
  tooltip?: string
}

// ── Theme ──
export type ThemeMode = 'dark' | 'light' | 'auto'

// ── Window (Tauri 预留) ──
export interface WindowCapabilities {
  minimize: () => void
  maximize: () => void
  close: () => void
  toggleFullscreen: () => void
  startDrag: () => void
}

export interface TrayCapabilities {
  show: () => void
  hide: () => void
  setTooltip: (tooltip: string) => void
}

export interface UpdateCapabilities {
  check: () => Promise<boolean>
  download: () => Promise<void>
  install: () => Promise<void>
  status: 'idle' | 'checking' | 'available' | 'downloading' | 'ready' | 'error'
}
