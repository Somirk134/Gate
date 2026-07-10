import { inject, type InjectionKey } from 'vue'
import type { AppContext } from '@/core/AppContext'
import { GateAppError } from '@/ipc'

export const APP_CONTEXT_KEY: InjectionKey<AppContext> = Symbol('APP_CONTEXT_KEY')

let currentContext: AppContext | null = null

export function setApplicationContext(context: AppContext) {
  currentContext = context
}

export function getApplicationContext() {
  if (!currentContext) {
    throw new GateAppError({
      code: 'APP_CONTEXT_NOT_INITIALIZED',
      messageKey: 'errors.application.contextNotInitialized',
      details: {},
      timestamp: Date.now(),
    })
  }

  return currentContext
}

export function tryGetApplicationContext() {
  return currentContext
}

export function useAppContext() {
  const context = inject(APP_CONTEXT_KEY, currentContext)

  if (!context) {
    throw new GateAppError({
      code: 'APP_CONTEXT_PROVIDER_MISSING',
      messageKey: 'errors.application.contextProviderMissing',
      details: {},
      timestamp: Date.now(),
    })
  }

  return context
}
