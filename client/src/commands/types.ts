import type { AppContext } from '@/core/AppContext'

export type CommandId = string

export interface CommandExecution<TArgs = unknown> {
  context: AppContext
  args?: TArgs
  signal?: AbortSignal
  source?: string
}

export type CommandHandler<TArgs = unknown, TResult = unknown> = (
  execution: CommandExecution<TArgs>,
) => TResult | Promise<TResult>

export interface Command<TArgs = unknown, TResult = unknown> {
  id: CommandId
  title: string
  titleKey?: string
  category: string
  categoryKey?: string
  description?: string
  descriptionKey?: string
  icon?: string
  shortcut?: string
  keywords?: string[]
  enabled?: (context: AppContext) => boolean
  handler: CommandHandler<TArgs, TResult>
}

export interface ExecuteCommandOptions<TArgs = unknown> {
  args?: TArgs
  signal?: AbortSignal
  source?: string
}
