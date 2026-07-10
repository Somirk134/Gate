import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export interface LayoutState {
  sidebarCollapsed: boolean
  sidebarHovered: boolean
  commandPaletteOpen: boolean
}

export const useLayoutStore = defineStore('layout', () => {
  // === State ===
  const sidebarCollapsed = ref(false)
  const sidebarHovered = ref(false)
  const commandPaletteOpen = ref(false)

  // === Getters ===
  const isSidebarVisible = computed(() => !sidebarCollapsed.value || sidebarHovered.value)
  const effectiveSidebarWidth = computed(() => (sidebarCollapsed.value ? 48 : 220))

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

  function openCommandPalette() {
    commandPaletteOpen.value = true
  }

  function closeCommandPalette() {
    commandPaletteOpen.value = false
  }

  function toggleCommandPalette() {
    commandPaletteOpen.value = !commandPaletteOpen.value
  }

  return {
    sidebarCollapsed,
    sidebarHovered,
    commandPaletteOpen,
    isSidebarVisible,
    effectiveSidebarWidth,
    toggleSidebar,
    expandSidebar,
    collapseSidebar,
    hoverSidebar,
    openCommandPalette,
    closeCommandPalette,
    toggleCommandPalette,
  }
})
