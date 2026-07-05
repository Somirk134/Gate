import type { EventBus } from "@/events/EventBus"
import type { AppEventMap } from "@/types/application"
import type { Disposable } from "@/utils/disposable"

export interface CacheOptions {
  ttl?: number
  namespace?: string
}

interface CacheEntry<T> {
  value: T
  createdAt: number
  expiresAt?: number
}

export interface CacheManager {
  get<T>(key: string, options?: CacheOptions): T | undefined
  set<T>(key: string, value: T, options?: CacheOptions): void
  has(key: string, options?: CacheOptions): boolean
  delete(key: string, options?: CacheOptions): void
  clear(namespace?: string): void
}

export interface DiskCacheManager extends CacheManager {
  readonly kind: "disk"
}

export class MemoryCacheManager implements CacheManager, Disposable {
  private readonly entries = new Map<string, CacheEntry<unknown>>()
  private readonly cleanupTimer?: number

  constructor(
    private readonly defaultNamespace = "memory",
    private readonly events?: EventBus<AppEventMap>,
    cleanupInterval = 60_000,
  ) {
    if (typeof window !== "undefined") {
      this.cleanupTimer = window.setInterval(() => this.cleanup(), cleanupInterval)
    }
  }

  get<T>(key: string, options: CacheOptions = {}): T | undefined {
    const cacheKey = this.toCacheKey(key, options.namespace)
    const entry = this.entries.get(cacheKey) as CacheEntry<T> | undefined

    if (!entry) {
      return undefined
    }

    if (this.isExpired(entry)) {
      this.delete(key, options)
      return undefined
    }

    return entry.value
  }

  set<T>(key: string, value: T, options: CacheOptions = {}) {
    const now = Date.now()
    this.entries.set(this.toCacheKey(key, options.namespace), {
      value,
      createdAt: now,
      expiresAt: options.ttl ? now + options.ttl : undefined,
    })
  }

  has(key: string, options: CacheOptions = {}) {
    return this.get(key, options) !== undefined
  }

  delete(key: string, options: CacheOptions = {}) {
    const namespace = options.namespace ?? this.defaultNamespace
    this.entries.delete(this.toCacheKey(key, namespace))
    void this.events?.publish("cache:expired", { namespace, key })
  }

  clear(namespace = this.defaultNamespace) {
    const prefix = `${namespace}:`

    for (const key of this.entries.keys()) {
      if (key.startsWith(prefix)) {
        this.entries.delete(key)
      }
    }
  }

  cleanup() {
    for (const [cacheKey, entry] of this.entries.entries()) {
      if (!this.isExpired(entry)) {
        continue
      }

      const [namespace, key] = cacheKey.split(":", 2)
      this.entries.delete(cacheKey)
      void this.events?.publish("cache:expired", { namespace, key })
    }
  }

  dispose() {
    if (this.cleanupTimer) {
      window.clearInterval(this.cleanupTimer)
    }

    this.entries.clear()
  }

  private toCacheKey(key: string, namespace = this.defaultNamespace) {
    return `${namespace}:${key}`
  }

  private isExpired(entry: CacheEntry<unknown>) {
    return Boolean(entry.expiresAt && entry.expiresAt <= Date.now())
  }
}
