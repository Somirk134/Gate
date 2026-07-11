import { onMounted } from 'vue'
import { SHORTCUT_SERVICE } from '@/services/tokens'
import { useService } from './useService'

export function useKeyboardShortcuts() {
  const shortcuts = useService(SHORTCUT_SERVICE)

  onMounted(() => {
    shortcuts.start()
  })

  return {
    shortcuts,
  }
}
