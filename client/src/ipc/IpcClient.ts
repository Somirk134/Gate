import type { Disposable } from "@/utils/disposable"

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

export class NoopIpcClient implements IpcClient {
  async invoke<TResult = unknown, TArgs = IpcPayload>(
    _command: string,
    _args?: TArgs,
  ): Promise<TResult> {
    throw new Error("IPC invoke is not connected in the application foundation layer.")
  }

  async listen<TPayload = unknown>(
    _event: string,
    _handler: (payload: IpcEvent<TPayload>) => void | Promise<void>,
  ): Promise<Disposable> {
    return {
      dispose: () => undefined,
    }
  }

  async emit<TPayload = unknown>(_event: string, _payload?: TPayload) {
    return undefined
  }

  async remove(_event: string) {
    return undefined
  }
}
