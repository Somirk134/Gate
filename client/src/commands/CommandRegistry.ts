import type { AppContext } from '@/core/AppContext'
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
      throw new Error(`Command already registered: ${command.id}`)
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
      throw new Error(`Command not found: ${id}`)
    }

    if (!this.context) {
      throw new Error(`Command registry has no application context: ${id}`)
    }

    if (command.enabled && !command.enabled(this.context)) {
      throw new Error(`Command is disabled: ${id}`)
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
