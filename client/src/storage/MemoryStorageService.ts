import type { StorageOptions, StorageService } from "./StorageService"

interface MemoryRecord<T> {
  value: T
  expiresAt?: number
}

export class MemoryStorageService implements StorageService {
  private readonly records = new Map<string, MemoryRecord<unknown>>()

  get<T>(key: string, options: StorageOptions<T> = {}): T | undefined {
    const storageKey = this.toStorageKey(key, options.namespace)
    const record = this.records.get(storageKey) as MemoryRecord<T> | undefined

    if (!record) {
      return undefined
    }

    if (record.expiresAt && record.expiresAt <= Date.now()) {
      this.records.delete(storageKey)
      return undefined
    }

    return record.value
  }

  set<T>(key: string, value: T, options: StorageOptions<T> = {}) {
    this.records.set(this.toStorageKey(key, options.namespace), {
      value,
      expiresAt: options.ttl ? Date.now() + options.ttl : undefined,
    })
  }

  remove(key: string, options: StorageOptions = {}) {
    this.records.delete(this.toStorageKey(key, options.namespace))
  }

  clear(namespace = "memory") {
    const prefix = `${namespace}:`

    for (const key of this.records.keys()) {
      if (key.startsWith(prefix)) {
        this.records.delete(key)
      }
    }
  }

  keys(namespace = "memory") {
    const prefix = `${namespace}:`

    return Array.from(this.records.keys())
      .filter((key) => key.startsWith(prefix))
      .map((key) => key.slice(prefix.length))
  }

  private toStorageKey(key: string, namespace = "memory") {
    return `${namespace}:${key}`
  }
}
