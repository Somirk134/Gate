import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export interface LayoutState {
    sidebarCollapsed: boolean
    sidebarHovered: boolean
    inspectorOpen: boolean
    inspectorWidth: number
    commandPaletteOpen: boolean
    globalSearchOpen: boolean
}

export const useLayoutStore = defineStore('layout', () => {
    // === State ===
    const sidebarCollapsed = ref(false)
    const sidebarHovered = ref(false)
    const inspectorOpen = ref(false)
    const inspectorWidth = ref(320)
    const commandPaletteOpen = ref(false)
    const globalSearchOpen = ref(false)

    // === Getters ===
    const isSidebarVisible = computed(() => !sidebarCollapsed.value || sidebarHovered.value)
    const isInspectorVisible = computed(() => inspectorOpen.value)
    const effectiveSidebarWidth = computed(() => sidebarCollapsed.value ? 48 : 220)

    // === Actions ===
    function toggleSidebar() {
        sidebarCollapsed.value = !sidebarCollapsed.value
    }

    function expandSidebar() {
        sidebarCollapsed.value = false
    }

    function collapseSidebar() {
        sidebarCollapsed.value = true
    }

    function hoverSidebar(hovered: boolean) {
        sidebarHovered.value = hovered
    }

    function toggleInspector() {
        inspectorOpen.value = !inspectorOpen.value
    }

    function openInspector() {
        inspectorOpen.value = true
    }

    function closeInspector() {
        inspectorOpen.value = false
    }

    function setInspectorWidth(width: number) {
        inspectorWidth.value = Math.max(240, Math.min(480, width))
    }

    function openCommandPalette() {
        commandPaletteOpen.value = true
    }

    function closeCommandPalette() {
        commandPaletteOpen.value = false
    }

    function toggleCommandPalette() {
        commandPaletteOpen.value = !commandPaletteOpen.value
    }

    function openGlobalSearch() {
        globalSearchOpen.value = true
    }

    function closeGlobalSearch() {
        globalSearchOpen.value = false
    }

    return {
        sidebarCollapsed,
        sidebarHovered,
        inspectorOpen,
        inspectorWidth,
        commandPaletteOpen,
        globalSearchOpen,
        isSidebarVisible,
        isInspectorVisible,
        effectiveSidebarWidth,
        toggleSidebar,
        expandSidebar,
        collapseSidebar,
        hoverSidebar,
        toggleInspector,
        openInspector,
        closeInspector,
        setInspectorWidth,
        openCommandPalette,
        closeCommandPalette,
        toggleCommandPalette,
        openGlobalSearch,
        closeGlobalSearch,
    }
})
