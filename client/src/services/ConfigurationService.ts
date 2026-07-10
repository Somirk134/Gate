import type { EventBus } from '@/events/EventBus'
import type { StorageOptions, StorageService } from '@/storage/StorageService'
import type { AppEventMap } from '@/types/application'
import type { Disposable } from '@/utils/disposable'

export interface AppConfiguration {
  appearance: {
    theme: 'dark' | 'light' | 'auto'
  }
  locale: string
  shortcuts: Record<string, string>
  window: {
    title: string
    bounds?: {
      width: number
      height: number
      x?: number
      y?: number
    }
  }
}

export type ConfigurationWatcher = (value: unknown, previousValue: unknown) => void | Promise<void>

export interface ConfigurationService {
  readonly defaults: AppConfiguration
  get<T = unknown>(key: string): T | undefined
  set<T = unknown>(key: string, value: T): void
  watch(key: string, watcher: ConfigurationWatcher): Disposable
  reset(key?: string): void
  snapshot(): AppConfiguration
}

export const defaultConfiguration: AppConfiguration = {
  appearance: {
    theme: 'dark',
  },
  locale: 'zh-CN',
  shortcuts: {
    'app.commandPalette.toggle': 'Ctrl+K',
    'settings.open': 'Ctrl+,',
  },
  window: {
    title: 'Gate',
  },
}

const configurationStorageOptions: StorageOptions<AppConfiguration> = {
  namespace: 'configuration',
  version: 2,
  cache: true,
  migrations: [
    {
      fromVersion: 1,
      toVersion: 2,
      migrate(value) {
        return {
          ...defaultConfiguration,
          ...value,
          appearance: {
            ...defaultConfiguration.appearance,
            ...value.appearance,
          },
          shortcuts: {
            ...defaultConfiguration.shortcuts,
            ...value.shortcuts,
          },
          window: {
            ...defaultConfiguration.window,
            ...value.window,
          },
          locale: value.locale === 'en' ? 'en-US' : value.locale || defaultConfiguration.locale,
        }
      },
    },
  ],
}

export class DefaultConfigurationService implements ConfigurationService {
  readonly defaults = defaultConfiguration

  private config: AppConfiguration
  private readonly watchers = new Map<string, Set<ConfigurationWatcher>>()

  constructor(
    private readonly storage: StorageService,
    private readonly events: EventBus<AppEventMap>,
  ) {
    this.config =
      this.storage.get<AppConfiguration>('app', configurationStorageOptions) ??
      this.clone(defaultConfiguration)
    if (this.config.locale === 'en') {
      this.config.locale = 'en-US'
      this.persist()
    }
  }

  get<T = unknown>(key: string): T | undefined {
    return this.readPath(this.config, key) as T | undefined
  }

  set<T = unknown>(key: string, value: T) {
    const previousValue = this.get(key)
    this.config = this.writePath(this.config, key, value)
    this.persist()
    this.notify(key, value, previousValue)
  }

  watch(key: string, watcher: ConfigurationWatcher): Disposable {
    const watchers = this.watchers.get(key) ?? new Set<ConfigurationWatcher>()
    watchers.add(watcher)
    this.watchers.set(key, watchers)

    return {
      dispose: () => {
        watchers.delete(watcher)
      },
    }
  }

  reset(key?: string) {
    if (!key) {
      const previousValue = this.config
      this.config = this.clone(defaultConfiguration)
      this.persist()
      this.notify('*', this.config, previousValue)
      return
    }

    const defaultValue = this.readPath(defaultConfiguration, key)
    this.set(key, defaultValue)
  }

  snapshot() {
    return this.clone(this.config)
  }

  private persist() {
    this.storage.set('app', this.config, configurationStorageOptions)
  }

  private notify(key: string, value: unknown, previousValue: unknown) {
    const exactWatchers = this.watchers.get(key) ?? new Set<ConfigurationWatcher>()
    const globalWatchers = this.watchers.get('*') ?? new Set<ConfigurationWatcher>()

    for (const watcher of [...exactWatchers, ...globalWatchers]) {
      void watcher(value, previousValue)
    }

    void this.events.publish('configuration:changed', {
      key,
      value,
      previousValue,
    })
  }

  private readPath(source: unknown, key: string): unknown {
    if (key === '*') {
      return source
    }

    return key.split('.').reduce<unknown>((current, segment) => {
      if (!this.isRecord(current)) {
        return undefined
      }

      return current[segment]
    }, source)
  }

  private writePath<T>(source: AppConfiguration, key: string, value: T): AppConfiguration {
    const next = this.clone(source) as unknown as Record<string, unknown>
    const segments = key.split('.')
    let cursor: Record<string, unknown> = next

    for (const segment of segments.slice(0, -1)) {
      const current = cursor[segment]
      cursor[segment] = this.isRecord(current) ? { ...current } : {}
      cursor = cursor[segment] as Record<string, unknown>
    }

    cursor[segments[segments.length - 1]] = value

    return next as unknown as AppConfiguration
  }

  private clone<T>(value: T): T {
    return JSON.parse(JSON.stringify(value)) as T
  }

  private isRecord(value: unknown): value is Record<string, unknown> {
    return Boolean(value && typeof value === 'object' && !Array.isArray(value))
  }
}
