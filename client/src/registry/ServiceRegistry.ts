import { GateAppError } from '@/ipc'
import type { Disposable } from '@/utils/disposable'

export interface ServiceToken<T> {
  readonly id: symbol
  readonly name: string
  readonly __type?: T
}

export type ServiceFactory<T> = (registry: ServiceRegistry) => T

export interface ServiceRegistrationOptions {
  eager?: boolean
  replace?: boolean
}

interface ServiceEntry<T> {
  token: ServiceToken<T>
  factory?: ServiceFactory<T>
  instance?: T
}

export function createServiceToken<T>(name: string): ServiceToken<T> {
  return {
    id: Symbol(name),
    name,
  }
}

export class ServiceRegistry implements Disposable {
  private readonly entries = new Map<symbol, ServiceEntry<unknown>>()

  register<T>(
    token: ServiceToken<T>,
    factory: ServiceFactory<T>,
    options: ServiceRegistrationOptions = {},
  ) {
    if (!options.replace && this.entries.has(token.id)) {
      throw createServiceError('SERVICE_ALREADY_REGISTERED', 'errors.application.serviceAlreadyRegistered', {
        name: token.name,
      })
    }

    const entry: ServiceEntry<T> = {
      token,
      factory,
    }

    this.entries.set(token.id, entry as ServiceEntry<unknown>)

    if (options.eager) {
      this.resolve(token)
    }
  }

  registerInstance<T>(
    token: ServiceToken<T>,
    instance: T,
    options: ServiceRegistrationOptions = {},
  ) {
    if (!options.replace && this.entries.has(token.id)) {
      throw createServiceError('SERVICE_ALREADY_REGISTERED', 'errors.application.serviceAlreadyRegistered', {
        name: token.name,
      })
    }

    this.entries.set(token.id, {
      token,
      instance,
    } as ServiceEntry<unknown>)
  }

  resolve<T>(token: ServiceToken<T>): T {
    const entry = this.entries.get(token.id) as ServiceEntry<T> | undefined

    if (!entry) {
      throw createServiceError('SERVICE_NOT_REGISTERED', 'errors.application.serviceNotRegistered', {
        name: token.name,
      })
    }

    if (entry.instance === undefined) {
      if (!entry.factory) {
        throw createServiceError('SERVICE_FACTORY_MISSING', 'errors.application.serviceFactoryMissing', {
          name: token.name,
        })
      }

      entry.instance = entry.factory(this)
    }

    return entry.instance
  }

  optional<T>(token: ServiceToken<T>): T | undefined {
    if (!this.entries.has(token.id)) {
      return undefined
    }

    return this.resolve(token)
  }

  has<T>(token: ServiceToken<T>) {
    return this.entries.has(token.id)
  }

  list() {
    return Array.from(this.entries.values()).map((entry) => entry.token.name)
  }

  async dispose() {
    const instances = Array.from(this.entries.values())
      .map((entry) => entry.instance)
      .filter((instance): instance is Disposable => {
        return Boolean(instance && typeof (instance as Disposable).dispose === 'function')
      })

    for (const instance of instances.reverse()) {
      await instance.dispose()
    }

    this.entries.clear()
  }
}

function createServiceError(
  code: string,
  messageKey: string,
  details: Record<string, unknown>,
) {
  return new GateAppError({
    code,
    messageKey,
    details,
    timestamp: Date.now(),
  })
}
