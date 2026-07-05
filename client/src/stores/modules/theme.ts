import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { tryGetApplicationContext } from '@/providers/appContext'
import { THEME_SERVICE } from '@/services/tokens'

export type ThemeMode = 'dark' | 'light' | 'auto'

export const useThemeStore = defineStore('theme', () => {
    // === State ===
    const mode = ref<ThemeMode>('dark')
    const systemPrefersDark = ref(false)
    let unsubscribeThemeChanged: (() => void) | null = null

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
        const service = tryGetApplicationContext()?.services.optional(THEME_SERVICE)

        if (service) {
            service.setTheme(newMode)
            syncFromService()
            return
        }

        mode.value = newMode
        syncSystemPreference()
    }

    function toggleTheme() {
        if (mode.value === 'dark') setTheme('light')
        else if (mode.value === 'light') setTheme('dark')
        else setTheme('dark')
    }

    function initTheme() {
        syncFromService()

        const context = tryGetApplicationContext()

        if (context && !unsubscribeThemeChanged) {
            unsubscribeThemeChanged = context.events.subscribe("theme:changed", () => {
                syncFromService()
            })
        }
    }

    function syncSystemPreference() {
        if (typeof window !== 'undefined') {
            systemPrefersDark.value = window.matchMedia('(prefers-color-scheme: dark)').matches
        }
    }

    function syncFromService() {
        const service = tryGetApplicationContext()?.services.optional(THEME_SERVICE)

        if (!service) {
            syncSystemPreference()
            return
        }

        const state = service.getState()
        mode.value = state.mode
        systemPrefersDark.value = state.systemPrefersDark
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
