import { ref, computed } from 'vue'
import type { GlobalTheme } from 'naive-ui'

type ThemeType = 'light' | 'dark'

const themeType = ref<ThemeType>('dark')

export function useTheme() {
  const theme = computed<GlobalTheme | null>(() => null)

  const toggleTheme = () => {
    themeType.value = themeType.value === 'light' ? 'dark' : 'light'
  }

  const setTheme = (type: ThemeType) => {
    themeType.value = type
  }

  const applyThemeClass = () => {
    document.documentElement.classList.remove('theme-light', 'theme-dark')
    document.documentElement.classList.add(`theme-${themeType.value}`)
  }

  return {
    theme,
    themeType,
    toggleTheme,
    setTheme,
    applyThemeClass,
  }
}
