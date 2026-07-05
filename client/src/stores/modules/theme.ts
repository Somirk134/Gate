import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export type ThemeMode = 'dark' | 'light' | 'auto'

export const useThemeStore = defineStore('theme', () => {
    // === State ===
    const mode = ref<ThemeMode>('dark')
    const systemPrefersDark = ref(false)

    // === Getters ===
    const effectiveTheme = computed<'light' | 'dark'>(() => {
        if (mode.value === 'auto') {
            return systemPrefersDark.value ? 'dark' : 'light'
        }
        return mode.value
    })

    const isDark = computed(() => effectiveTheme.value === 'dark')
    const isLight = computed(() => effectiveTheme.value === 'light')
    const currentMode = computed(() => mode.value)

    // === Actions ===
    function setTheme(newMode: ThemeMode) {
        mode.value = newMode
        persist()
    }

    function toggleTheme() {
        if (mode.value === 'dark') setTheme('light')
        else if (mode.value === 'light') setTheme('dark')
        else setTheme('dark')
    }

    function initTheme() {
        const saved = localStorage.getItem('gate-theme') as ThemeMode | null
        if (saved && ['dark', 'light', 'auto'].includes(saved)) {
            mode.value = saved
        }
        syncSystemPreference()
        if (typeof window !== 'undefined') {
            const mql = window.matchMedia('(prefers-color-scheme: dark)')
            mql.addEventListener?.('change', (e) => {
                systemPrefersDark.value = e.matches
            })
        }
    }

    function syncSystemPreference() {
        if (typeof window !== 'undefined') {
            systemPrefersDark.value = window.matchMedia('(prefers-color-scheme: dark)').matches
        }
    }

    function persist() {
        localStorage.setItem('gate-theme', mode.value)
    }

    return {
        mode,
        effectiveTheme,
        isDark,
        isLight,
        currentMode,
        setTheme,
        toggleTheme,
        initTheme,
    }
})
