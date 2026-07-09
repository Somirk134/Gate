import { inject, type InjectionKey } from 'vue'
import type { AppContext } from '@/core/AppContext'

export const APP_CONTEXT_KEY: InjectionKey<AppContext> = Symbol('APP_CONTEXT_KEY')

let currentContext: AppContext | null = null

export function setApplicationContext(context: AppContext) {
  currentContext = context
}

export function getApplicationContext() {
  if (!currentContext) {
    throw new Error('Application context has not been initialized.')
  }

  return currentContext
}

export function tryGetApplicationContext() {
  return currentContext
}

export function useAppContext() {
  const context = inject(APP_CONTEXT_KEY, currentContext)

  if (!context) {
    throw new Error('Application context provider is missing.')
  }

  return context
}
