import type { CacheManager } from '@/cache/CacheManager'
import { createServiceToken } from '@/registry/ServiceRegistry'
import type { ErrorHandler } from '@/errors/ErrorHandler'
import type { IpcClient } from '@/ipc'
import type { LoggerService } from '@/logger/LoggerService'
import type { PluginManager } from '@/plugins/PluginManager'
import type { RequestClient } from '@/network'
import type { StorageService } from '@/storage/StorageService'
import type { ThemeService } from '@/theme/ThemeService'
import type { WindowService } from '@/window/WindowService'
import type { AutoUpdateService } from '@/updates/UpdateService'
import type { ConfigurationService } from './ConfigurationService'
import type { DialogService } from './DialogService'
import type { NotificationService } from './NotificationService'
import type { ShortcutService } from './ShortcutService'

export const LOGGER_SERVICE = createServiceToken<LoggerService>('logger')
export const STORAGE_SERVICE = createServiceToken<StorageService>('storage')
export const CACHE_SERVICE = createServiceToken<CacheManager>('cache')
export const CONFIGURATION_SERVICE = createServiceToken<ConfigurationService>('configuration')
export const WINDOW_SERVICE = createServiceToken<WindowService>('window')
export const THEME_SERVICE = createServiceToken<ThemeService>('theme')
export const SHORTCUT_SERVICE = createServiceToken<ShortcutService>('shortcut')
export const NOTIFICATION_SERVICE = createServiceToken<NotificationService>('notification')
export const DIALOG_SERVICE = createServiceToken<DialogService>('dialog')
export const IPC_SERVICE = createServiceToken<IpcClient>('ipc')
export const REQUEST_SERVICE = createServiceToken<RequestClient>('request')
export const UPDATE_SERVICE = createServiceToken<AutoUpdateService>('update')
export const PLUGIN_MANAGER_SERVICE = createServiceToken<PluginManager>('plugin-manager')
export const ERROR_HANDLER_SERVICE = createServiceToken<ErrorHandler>('error-handler')
