import { onMounted, onUnmounted } from 'vue'
import { useLayoutStore } from '@stores'

/* ==================================================================
   useKeyboardShortcuts — 全局键盘快捷键
   职责：注册全局快捷键，分发给各 Shell 模块
   ================================================================== */

export function useKeyboardShortcuts() {
  const layoutStore = useLayoutStore()
  function handleKeydown(e: KeyboardEvent) {
    const mod = e.metaKey || e.ctrlKey

    // Toggle Sidebar: Ctrl + \
    if (mod && e.key === '\\') {
      e.preventDefault()
      layoutStore.toggleSidebar()
      return
    }

    // Toggle Inspector: Ctrl + Shift + I
    if (mod && e.shiftKey && (e.key === 'i' || e.key === 'I')) {
      e.preventDefault()
      layoutStore.toggleInspector()
      return
    }

    // Command Palette: Ctrl + K
    if (mod && (e.key === 'k' || e.key === 'K') && !e.shiftKey) {
      e.preventDefault()
      if (layoutStore.commandPaletteOpen) {
        layoutStore.closeCommandPalette()
      } else {
        layoutStore.openCommandPalette()
      }
      return
    }

    // Global Search: Ctrl + Shift + K
    if (mod && e.shiftKey && (e.key === 'k' || e.key === 'K')) {
      e.preventDefault()
      if (layoutStore.globalSearchOpen) {
        layoutStore.closeGlobalSearch()
      } else {
        layoutStore.openGlobalSearch()
      }
      return
    }

    // Close overlays on Escape
    if (e.key === 'Escape') {
      if (layoutStore.commandPaletteOpen) {
        layoutStore.closeCommandPalette()
        return
      }
      if (layoutStore.globalSearchOpen) {
        layoutStore.closeGlobalSearch()
        return
      }
      return
    }
  }

  onMounted(() => {
    document.addEventListener('keydown', handleKeydown)
  })

  onUnmounted(() => {
    document.removeEventListener('keydown', handleKeydown)
  })

  return {
    handleKeydown,
  }
}
