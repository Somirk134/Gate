import type { Disposable } from "@/utils/disposable"
import { emit as tauriEmit, listen as tauriListen } from "@tauri-apps/api/event"
import { invoke as tauriInvoke } from "@tauri-apps/api/core"

export type IpcPayload = Record<string, unknown> | undefined

export interface IpcEvent<TPayload = unknown> {
  name: string
  payload: TPayload
}

export interface IpcClient {
  invoke<TResult = unknown, TArgs = IpcPayload>(
    command: string,
    args?: TArgs,
  ): Promise<TResult>
  listen<TPayload = unknown>(
    event: string,
    handler: (payload: IpcEvent<TPayload>) => void | Promise<void>,
  ): Promise<Disposable>
  emit<TPayload = unknown>(event: string, payload?: TPayload): Promise<void>
  remove(event: string): Promise<void>
}

export class TauriIpcClient implements IpcClient {
  async invoke<TResult = unknown, TArgs = IpcPayload>(
    command: string,
    args?: TArgs,
  ): Promise<TResult> {
    return tauriInvoke<TResult>(command, args as Record<string, unknown> | undefined)
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
