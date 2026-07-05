<template>
    <n-config-provider
        :theme="naiveTheme"
        :theme-overrides="themeOverrides"
        :locale="localeStore.naiveLocale"
        :date-locale="localeStore.naiveDateLocale"
    >
        <n-message-provider placement="top" :max="5">
            <n-dialog-provider>
                <n-notification-provider placement="top-right" :max="8">
                    <n-loading-bar-provider>
                        <ThemeProvider>
                            <slot />
                        </ThemeProvider>
                    </n-loading-bar-provider>
                </n-notification-provider>
            </n-dialog-provider>
        </n-message-provider>
    </n-config-provider>
</template>

<script setup lang="ts">
import { computed } from "vue"
import { darkTheme } from "naive-ui"
import { useThemeStore } from "@stores"
import ThemeProvider from "./ThemeProvider.vue"
import { useLocaleSwitcher } from "@composables/useLocaleSwitcher"

const themeStore = useThemeStore()
const { localeStore } = useLocaleSwitcher()

const naiveTheme = computed(() => {
    return themeStore.isDark ? darkTheme : null
})

const themeOverrides = {
    common: {
        primaryColor: "#5B8DEF",
        primaryColorHover: "#7BA4F4",
        primaryColorPressed: "#4A7AD4",
        primaryColorSuppl: "#7C6FF2",
        bodyColor: "#0D0D0F",
        cardColor: "#1A1A1E",
        modalColor: "#252529",
        popoverColor: "#252529",
        borderColor: "#2A2A30",
        dividerColor: "#1F1F25",
        inputColor: "#151519",
        textColor1: "#EDEDEF",
        textColor2: "#9D9DA3",
        textColor3: "#6B6B72",
        textColorDisabled: "#4A4A50",
        fontSize: "13px",
        fontSizeSmall: "12px",
        fontSizeMini: "11px",
        borderRadius: "8px",
        borderRadiusSmall: "6px",
        heightSmall: "28px",
        heightMedium: "32px",
        heightLarge: "36px",
    },
}
</script>
