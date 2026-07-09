<template>
  <div
    class="app-shell"
    :class="{
      'sidebar-collapsed': layout.sidebarCollapsed,
      'inspector-open': layout.inspectorOpen,
    }"
  >
    <!-- Sidebar -->
    <AppSidebar />

    <!-- Main Area -->
    <div class="main-area">
      <AppHeader />
      <AppContent />
      <AppStatusBar />
    </div>

    <!-- Inspector -->
    <AppInspector v-if="layout.inspectorOpen" />

    <!-- Layers -->
    <AppCommandPalette v-if="layout.commandPaletteOpen" />
    <AppNotificationLayer />
    <AppDialogLayer />
    <AppLoadingLayer v-if="loading.isLoading" />
    <AppWelcomeOverlay />
  </div>
</template>

<script setup lang="ts">
import { computed } from "vue"
import { useLayoutStore, useLoadingStore } from "@stores"
import AppSidebar from "./AppSidebar.vue"
import AppHeader from "./AppHeader.vue"
import AppContent from "./AppContent.vue"
import AppInspector from "./AppInspector.vue"
import AppStatusBar from "./AppStatusBar.vue"
import AppCommandPalette from "./AppCommandPalette.vue"
import AppNotificationLayer from "./AppNotificationLayer.vue"
import AppDialogLayer from "./AppDialogLayer.vue"
import AppLoadingLayer from "./AppLoadingLayer.vue"
import AppWelcomeOverlay from "./AppWelcomeOverlay.vue"

const layout = useLayoutStore()
const loading = useLoadingStore()

const inspectorWidth = computed(() => layout.inspectorWidth + 'px')

</script>

<style scoped>
.app-shell {
    display: grid;
    grid-template-columns: var(--sidebar-width) 1fr;
    height: 100vh;
    overflow: hidden;
    transition: grid-template-columns var(--duration-standard) var(--ease-out);
}

.app-shell.sidebar-collapsed {
    grid-template-columns: var(--sidebar-collapsed-width) 1fr;
}

.app-shell.inspector-open {
    grid-template-columns: var(--sidebar-width) 1fr v-bind(inspectorWidth);
}

.app-shell.sidebar-collapsed.inspector-open {
    grid-template-columns: var(--sidebar-collapsed-width) 1fr v-bind(inspectorWidth);
}

.main-area {
    display: flex;
    flex-direction: column;
    min-width: 0;
    overflow: hidden;
}
</style>
