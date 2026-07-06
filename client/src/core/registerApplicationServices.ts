import type { AppContext } from "./AppContext"
import { BrowserShortcutService } from "@/services/ShortcutService"
import { BrowserWindowService } from "@/window/WindowService"
import { DefaultConfigurationService } from "@/services/ConfigurationService"
import { DefaultLoggerService } from "@/logger/LoggerService"
import { DefaultPluginManager } from "@/plugins/PluginManager"
import { DefaultThemeService } from "@/theme/ThemeService"
import { EventDialogService } from "@/services/DialogService"
import { EventNotificationService } from "@/services/NotificationService"
import { GlobalErrorHandler } from "@/errors/ErrorHandler"
import { LocalStorageService } from "@/storage/StorageService"
import { MemoryCacheManager } from "@/cache/CacheManager"
import { TauriIpcClient } from "@/ipc"
import { FetchRequestClient } from "@/network"
import { StubAutoUpdateService } from "@/updates"
import { createPluginAPI } from "@/plugins/PluginAPI"
import {
  CACHE_SERVICE,
  CONFIGURATION_SERVICE,
  DIALOG_SERVICE,
  ERROR_HANDLER_SERVICE,
  IPC_SERVICE,
  LOGGER_SERVICE,
  NOTIFICATION_SERVICE,
  PLUGIN_MANAGER_SERVICE,
  REQUEST_SERVICE,
  SHORTCUT_SERVICE,
  STORAGE_SERVICE,
  THEME_SERVICE,
  UPDATE_SERVICE,
  WINDOW_SERVICE,
} from "@/services/tokens"

export function registerApplicationServices(context: AppContext) {
  const { services, events, commands } = context

  services.register(LOGGER_SERVICE, () => new DefaultLoggerService(), { eager: true })
  services.register(STORAGE_SERVICE, () => new LocalStorageService("gate", events), { eager: true })
  services.register(
    CONFIGURATION_SERVICE,
    (registry) => new DefaultConfigurationService(registry.resolve(STORAGE_SERVICE), events),
    { eager: true },
  )
  services.register(
    CACHE_SERVICE,
    () => new MemoryCacheManager("application", events),
    { eager: true },
  )
  services.register(NOTIFICATION_SERVICE, () => new EventNotificationService(events), { eager: true })
  services.register(DIALOG_SERVICE, () => new EventDialogService(events), { eager: true })
  services.register(
    THEME_SERVICE,
    (registry) => new DefaultThemeService(registry.resolve(CONFIGURATION_SERVICE), events),
    { eager: true },
  )
  services.register(
    WINDOW_SERVICE,
    (registry) => new BrowserWindowService(registry.resolve(CONFIGURATION_SERVICE)),
    { eager: true },
  )
  services.register(
    SHORTCUT_SERVICE,
    (registry) => new BrowserShortcutService(
      commands,
      events,
      registry.resolve(LOGGER_SERVICE).child("shortcuts"),
    ),
    { eager: true },
  )
  services.register(IPC_SERVICE, () => new TauriIpcClient(), { eager: true })
  services.register(REQUEST_SERVICE, () => new FetchRequestClient(), { eager: true })
  services.register(UPDATE_SERVICE, () => new StubAutoUpdateService(events), { eager: true })
  services.register(
    PLUGIN_MANAGER_SERVICE,
    () => new DefaultPluginManager(createPluginAPI(context), events),
    { eager: true },
  )
  services.register(
    ERROR_HANDLER_SERVICE,
    (registry) => new GlobalErrorHandler(
      registry.resolve(LOGGER_SERVICE).child("errors"),
      events,
      registry.resolve(NOTIFICATION_SERVICE),
    ),
    { eager: true },
  )
}
