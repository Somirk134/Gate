<template>
    <div class="desktop-layout">
        <AppShell />
    </div>
</template>

<script setup lang="ts">
import AppShell from "@/shell/AppShell.vue"
import { useKeyboardShortcuts } from "@composables/useKeyboardShortcuts"
import { useNavigationStore } from "@stores"
import { watch } from "vue"
import { useRoute } from "vue-router"

// 注册全局快捷键
useKeyboardShortcuts()

// 同步路由到导航 Store
const route = useRoute()
const navStore = useNavigationStore()
watch(
    () => route,
    (r) => {
        navStore.setCurrentRoute(r)
    },
    { immediate: true, deep: true }
)
</script>

<style scoped>
.desktop-layout {
    width: 100%;
    height: 100%;
    overflow: hidden;
}
</style>
