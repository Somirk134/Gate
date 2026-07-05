<template>
    <div class="theme-provider" :class="themeClass">
        <slot />
    </div>
</template>

<script setup lang="ts">
import { computed, watch, onMounted } from "vue"
import { useThemeStore } from "@stores"

const themeStore = useThemeStore()

const themeClass = computed(() => {
    if (themeStore.effectiveTheme === 'light') return 'theme-light'
    return 'theme-dark'
})

watch(
    () => themeStore.effectiveTheme,
    (theme) => {
        document.documentElement.classList.remove('theme-dark', 'theme-light')
        document.documentElement.classList.add(theme === 'light' ? 'theme-light' : 'theme-dark')
    },
    { immediate: true }
)

onMounted(() => {
    themeStore.initTheme()
})
</script>

<style scoped>
.theme-provider {
    width: 100%;
    height: 100%;
}
</style>
