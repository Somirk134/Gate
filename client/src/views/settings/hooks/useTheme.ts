import { watch } from 'vue'
import { useThemeStore } from '@stores'
import { useSettingsStore } from '../stores'

export function useTheme() {
  const settingsStore = useSettingsStore()
  const themeStore = useThemeStore()

  const stop = watch(
    () => settingsStore.read('appearance.theme'),
    (value) => {
      if (value === 'light') {
        themeStore.setTheme('light')
        return
      }

      if (value === 'system') {
        themeStore.setTheme('auto')
        return
      }

      themeStore.setTheme('dark')
    },
    { immediate: true },
  )

  return {
    stop,
    themeStore,
  }
}
