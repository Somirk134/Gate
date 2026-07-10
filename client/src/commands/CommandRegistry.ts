import type { AppContext } from '@/core/AppContext'
import { GateAppError } from '@/ipc'
import type { Disposable } from '@/utils/disposable'
import type { Command, CommandId, ExecuteCommandOptions } from './types'

export class CommandRegistry implements Disposable {
  private readonly commands = new Map<CommandId, Command>()
  private context: AppContext | null = null

  bindContext(context: AppContext) {
    this.context = context
  }

  register<TArgs = unknown, TResult = unknown>(command: Command<TArgs, TResult>): Disposable {
    if (this.commands.has(command.id)) {
      throw createCommandError('COMMAND_ALREADY_REGISTERED', 'errors.application.commandAlreadyRegistered', {
        id: command.id,
      })
    }

    this.commands.set(command.id, command as Command)

    return {
      dispose: () => {
        this.unregister(command.id)
      },
    }
  }

  unregister(id: CommandId) {
    this.commands.delete(id)
  }

  has(id: CommandId) {
    return this.commands.has(id)
  }

  get(id: CommandId) {
    return this.commands.get(id)
  }

  list() {
    return Array.from(this.commands.values()).sort((a, b) => {
      return a.title.localeCompare(b.title)
    })
  }

  async execute<TResult = unknown, TArgs = unknown>(
    id: CommandId,
    options: ExecuteCommandOptions<TArgs> = {},
  ): Promise<TResult> {
    const command = this.commands.get(id) as Command<TArgs, TResult> | undefined

    if (!command) {
      throw createCommandError('COMMAND_NOT_FOUND', 'errors.application.commandNotFound', { id })
    }

    if (!this.context) {
      throw createCommandError('COMMAND_CONTEXT_MISSING', 'errors.application.commandContextMissing', {
        id,
      })
    }

    if (command.enabled && !command.enabled(this.context)) {
      throw createCommandError('COMMAND_DISABLED', 'errors.application.commandDisabled', { id })
    }

    try {
      const result = await command.handler({
        context: this.context,
        args: options.args,
        signal: options.signal,
        source: options.source,
      })

      await this.context.events.publish('command:executed', {
        id,
        source: options.source,
      })

      return result
    } catch (error) {
      await this.context.events.publish('command:failed', {
        id,
        source: options.source,
        error,
      })
      throw error
    }
  }

  dispose() {
    this.commands.clear()
    this.context = null
  }
}

function createCommandError(
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
