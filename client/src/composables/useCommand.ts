import { computed } from "vue"
import { useAppContext } from "@/providers/appContext"
import type { Command, ExecuteCommandOptions } from "@/commands/types"

export function useCommand() {
  const context = useAppContext()

  async function execute<TResult = unknown, TArgs = unknown>(
    id: string,
    options?: ExecuteCommandOptions<TArgs>,
  ) {
    return context.commands.execute<TResult, TArgs>(id, options)
  }

  function register<TArgs = unknown, TResult = unknown>(
    command: Command<TArgs, TResult>,
  ) {
    return context.commands.register(command)
  }

  return {
    commands: computed(() => context.commands.list()),
    execute,
    register,
  }
}
