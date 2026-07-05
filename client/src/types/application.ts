import type { AppLifecycleTransition } from "@/core/lifecycle"

export type AppRuntime = "desktop" | "browser" | "test"

export interface AppEnvironment {
  name: string
  version: string
  runtime: AppRuntime
  dev: boolean
}

export interface AppErrorPayload {
  error: unknown
  message: string
  fatal: boolean
  context?: string
}

export interface ConfigurationChangedPayload {
  key: string
  value: unknown
  previousValue: unknown
}

export interface StorageChangedPayload {
  namespace: string
  key: string
  action: "set" | "remove" | "clear" | "migrate" | "expire"
}

export interface ThemeChangedPayload {
  mode: "dark" | "light" | "auto"
  effectiveTheme: "dark" | "light"
}

export interface ShortcutTriggeredPayload {
  id: string
  commandId: string
  shortcut: string
}

export interface NotificationPayload {
  id?: string
  type: "success" | "error" | "warning" | "info"
  title: string
  content?: string
  duration?: number
  closable?: boolean
}

export interface DialogPayload {
  id?: string
  type: "modal" | "confirm" | "alert" | "delete" | "form"
  title: string
  content?: string
  props?: Record<string, unknown>
}

export interface AppEventMap {
  "app:ready": { at: number }
  "app:error": AppErrorPayload
  "cache:expired": { namespace: string; key: string }
  "command-palette:close": undefined
  "command-palette:open": undefined
  "command-palette:toggle": undefined
  "command:executed": { id: string; source?: string }
  "command:failed": { id: string; source?: string; error: unknown }
  "command:reserved": { id: string; args?: unknown }
  "configuration:changed": ConfigurationChangedPayload
  "dialog:show": DialogPayload
  "lifecycle:transition": AppLifecycleTransition
  "navigation:request": { path: string; replace?: boolean }
  "notification:show": NotificationPayload
  "global-search:toggle": undefined
  "inspector:toggle": undefined
  "plugin:activated": { id: string }
  "plugin:deactivated": { id: string }
  "sidebar:toggle": undefined
  "shortcut:triggered": ShortcutTriggeredPayload
  "storage:changed": StorageChangedPayload
  "theme:changed": ThemeChangedPayload
  "update:status": { status: string }
}
