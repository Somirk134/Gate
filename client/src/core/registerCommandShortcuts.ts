import type { AppContext } from './AppContext'
import { SHORTCUT_SERVICE } from '@/services/tokens'

export function registerCommandShortcuts(context: AppContext) {
  const shortcutService = context.services.resolve(SHORTCUT_SERVICE)
  const shortcutConfig = context.configuration.get<Record<string, string>>('shortcuts') ?? {}
  const usedShortcuts = new Set(
    shortcutService.list().map((binding) => normalizeShortcut(binding.shortcut)),
  )

  for (const command of context.commands.list()) {
    if (!command.shortcut) {
      continue
    }

    const bindingId = `shortcut.${command.id}`
    if (shortcutService.list().some((binding) => binding.id === bindingId)) {
      continue
    }

    const shortcut = shortcutConfig[command.id] ?? command.shortcut
    const normalized = normalizeShortcut(shortcut)
    if (usedShortcuts.has(normalized)) {
      continue
    }

    shortcutService.register({
      id: bindingId,
      commandId: command.id,
      shortcut,
    })
    usedShortcuts.add(normalized)
  }
}

function normalizeShortcut(shortcut: string) {
  return shortcut.trim().toLowerCase()
}
