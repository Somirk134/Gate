import type { Disposable } from '@/utils/disposable'
import { emit as tauriEmit, listen as tauriListen } from '@tauri-apps/api/event'
import { invoke as tauriInvoke } from '@tauri-apps/api/core'
import { i18n } from '@/i18n'

export type IpcPayload = Record<string, unknown> | undefined

export interface IpcEvent<TPayload = unknown> {
  name: string
  payload: TPayload
}

export interface IpcClient {
  invoke<TResult = unknown, TArgs = IpcPayload>(command: string, args?: TArgs): Promise<TResult>
  listen<TPayload = unknown>(
    event: string,
    handler: (payload: IpcEvent<TPayload>) => void | Promise<void>,
  ): Promise<Disposable>
  emit<TPayload = unknown>(event: string, payload?: TPayload): Promise<void>
  remove(event: string): Promise<void>
}

export interface AppErrorModel {
  code: string
  messageKey: string
  details?: Record<string, unknown>
  timestamp: number
}

export class GateAppError extends Error implements AppErrorModel {
  code: string
  messageKey: string
  details: Record<string, unknown>
  timestamp: number

  constructor(model: AppErrorModel) {
    super(localizeErrorMessage(model.messageKey, model.details))
    this.name = 'GateAppError'
    this.code = model.code
    this.messageKey = model.messageKey
    this.details = model.details ?? {}
    this.timestamp = model.timestamp
  }
}

export class TauriIpcClient implements IpcClient {
  async invoke<TResult = unknown, TArgs = IpcPayload>(
    command: string,
    args?: TArgs,
  ): Promise<TResult> {
    try {
      return await tauriInvoke<TResult>(command, args as Record<string, unknown> | undefined)
    } catch (error) {
      throw normalizeIpcError(error, command)
    }
  }

  async listen<TPayload = unknown>(
    event: string,
    handler: (payload: IpcEvent<TPayload>) => void | Promise<void>,
  ): Promise<Disposable> {
    const unlisten = await tauriListen<TPayload>(event, async (payload) => {
      await handler({
        name: event,
        payload: payload.payload,
      })
    })

    return {
      dispose: () => {
        unlisten()
      },
    }
  }

  async emit<TPayload = unknown>(event: string, payload?: TPayload) {
    await tauriEmit(event, payload)
  }

  async remove(_event: string) {
    return undefined
  }
}

export function normalizeIpcError(error: unknown, command = 'unknown'): GateAppError {
  const parsed = parseStructuredError(error)
  if (parsed) {
    return new GateAppError(parsed)
  }

  return new GateAppError({
    code: 'LEGACY_IPC_ERROR',
    messageKey: 'errors.legacyIpc',
    details: {
      command,
      source: toSourceMessage(error),
    },
    timestamp: Date.now(),
  })
}

function parseStructuredError(error: unknown): AppErrorModel | null {
  if (isAppErrorModel(error)) return error
  if (typeof error !== 'string') return null

  try {
    const parsed = JSON.parse(error) as unknown
    return isAppErrorModel(parsed) ? parsed : null
  } catch {
    return null
  }
}

function isAppErrorModel(value: unknown): value is AppErrorModel {
  if (!isRecord(value)) return false
  return (
    typeof value.code === 'string' &&
    typeof value.messageKey === 'string' &&
    typeof value.timestamp === 'number'
  )
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null
}

function toSourceMessage(error: unknown) {
  if (error instanceof Error) return error.message
  if (typeof error === 'string') return error
  return String(error)
}

function localizeErrorMessage(messageKey: string, details: Record<string, unknown> = {}) {
  try {
    const global = i18n.global as unknown as {
      t: (key: string, named?: Record<string, unknown>) => string
    }
    const translated = global.t(messageKey, details)
    return translated && translated !== messageKey ? translated : messageKey
  } catch {
    return messageKey
  }
}
