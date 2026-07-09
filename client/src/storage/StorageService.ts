import type { EventBus } from '@/events/EventBus'
import type { AppEventMap } from '@/types/application'

export interface StorageMigration<T = unknown> {
  fromVersion: number
  toVersion: number
  migrate(value: T): T
}

export interface StorageOptions<T = unknown> {
  namespace?: string
  version?: number
  ttl?: number
  cache?: boolean
  migrations?: StorageMigration<T>[]
}

interface StorageRecord<T> {
  value: T
  version: number
  createdAt: number
  updatedAt: number
  expiresAt?: number
}

export interface StorageService {
  get<T>(key: string, options?: StorageOptions<T>): T | undefined
  set<T>(key: string, value: T, options?: StorageOptions<T>): void
  remove(key: string, options?: StorageOptions): void
  clear(namespace?: string): void
  keys(namespace?: string): string[]
}

export class LocalStorageService implements StorageService {
  private readonly memory = new Map<string, StorageRecord<unknown>>()

  constructor(
    private readonly defaultNamespace = 'gate',
    private readonly events?: EventBus<AppEventMap>,
  ) {}

  get<T>(key: string, options: StorageOptions<T> = {}): T | undefined {
    const storageKey = this.toStorageKey(key, options.namespace)
    const cached = this.memory.get(storageKey) as StorageRecord<T> | undefined

    if (options.cache && cached && !this.isExpired(cached)) {
      return cached.value
    }

    const record = this.readRecord<T>(storageKey)

    if (!record) {
      return undefined
    }

    if (this.isExpired(record)) {
      this.remove(key, options)
      void this.events?.publish('storage:changed', {
        namespace: options.namespace ?? this.defaultNamespace,
        key,
        action: 'expire',
      })
      return undefined
    }

    const version = options.version ?? record.version
    const migrated = this.migrate(record.value, record.version, version, options.migrations)

    if (migrated.migrated) {
      this.set(key, migrated.value, {
        ...options,
        version,
      })
      void this.events?.publish('storage:changed', {
        namespace: options.namespace ?? this.defaultNamespace,
        key,
        action: 'migrate',
      })
    }

    this.memory.set(storageKey, {
      ...record,
      value: migrated.value,
      version,
    })

    return migrated.value
  }

  set<T>(key: string, value: T, options: StorageOptions<T> = {}) {
    const storageKey = this.toStorageKey(key, options.namespace)
    const now = Date.now()
    const record: StorageRecord<T> = {
      value,
      version: options.version ?? 1,
      createdAt: now,
      updatedAt: now,
      expiresAt: options.ttl ? now + options.ttl : undefined,
    }

    this.memory.set(storageKey, record)
    this.storage?.setItem(storageKey, JSON.stringify(record))
    void this.events?.publish('storage:changed', {
      namespace: options.namespace ?? this.defaultNamespace,
      key,
      action: 'set',
    })
  }

  remove(key: string, options: StorageOptions = {}) {
    const storageKey = this.toStorageKey(key, options.namespace)
    this.memory.delete(storageKey)
    this.storage?.removeItem(storageKey)
    void this.events?.publish('storage:changed', {
      namespace: options.namespace ?? this.defaultNamespace,
      key,
      action: 'remove',
    })
  }

  clear(namespace = this.defaultNamespace) {
    for (const key of this.keys(namespace)) {
      const storageKey = this.toStorageKey(key, namespace)
      this.memory.delete(storageKey)
      this.storage?.removeItem(storageKey)
    }

    void this.events?.publish('storage:changed', {
      namespace,
      key: '*',
      action: 'clear',
    })
  }

  keys(namespace = this.defaultNamespace) {
    const prefix = `${namespace}:`
    const keys: string[] = []

    if (!this.storage) {
      return keys
    }

    for (let index = 0; index < this.storage.length; index += 1) {
      const key = this.storage.key(index)

      if (key?.startsWith(prefix)) {
        keys.push(key.slice(prefix.length))
      }
    }

    return keys
  }

  private get storage(): Storage | undefined {
    if (typeof window === 'undefined') {
      return undefined
    }

    return window.localStorage
  }

  private readRecord<T>(storageKey: string): StorageRecord<T> | undefined {
    const raw = this.storage?.getItem(storageKey)

    if (!raw) {
      return undefined
    }

    try {
      return JSON.parse(raw) as StorageRecord<T>
    } catch {
      this.storage?.removeItem(storageKey)
      return undefined
    }
  }

  private toStorageKey(key: string, namespace = this.defaultNamespace) {
    return `${namespace}:${key}`
  }

  private isExpired(record: StorageRecord<unknown>) {
    return Boolean(record.expiresAt && record.expiresAt <= Date.now())
  }

  private migrate<T>(
    value: T,
    currentVersion: number,
    targetVersion: number,
    migrations: StorageMigration<T>[] = [],
  ) {
    if (currentVersion === targetVersion) {
      return { value, migrated: false }
    }

    let nextValue = value
    let version = currentVersion
    const sorted = [...migrations].sort((a, b) => a.fromVersion - b.fromVersion)

    for (const migration of sorted) {
      if (migration.fromVersion === version && migration.toVersion <= targetVersion) {
        nextValue = migration.migrate(nextValue)
        version = migration.toVersion
      }
    }

    return {
      value: nextValue,
      migrated: version !== currentVersion,
    }
  }
}
