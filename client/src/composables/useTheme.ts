import { computed } from 'vue'
import { useAppStore } from '@stores'

export function useTheme() {
  const appStore = useAppStore()

  const isDark = computed(() => appStore.theme === 'dark')

  function toggleTheme() {
    appStore.setTheme(isDark.value ? 'light' : 'dark')
  }

  return {
    isDark,
    theme: computed(() => appStore.theme),
    toggleTheme,
  }
}
