import { SHORTCUT_SERVICE } from '@/services/tokens'
import { useService } from './useService'

export function useKeyboardShortcuts() {
  const shortcuts = useService(SHORTCUT_SERVICE)

  return {
    shortcuts,
    handleKeydown: (_event: KeyboardEvent) => undefined,
  }
}
