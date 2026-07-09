<template>
  <n-config-provider
    :theme="naiveTheme"
    :theme-overrides="themeOverrides"
    :locale="localeStore.naiveLocale"
    :date-locale="localeStore.naiveDateLocale">
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
import { computed, onMounted, onUnmounted, provide } from 'vue'
import { darkTheme } from 'naive-ui'
import { useDialogStore, useLayoutStore, useNotificationStore, useThemeStore } from '@stores'
import ThemeProvider from './ThemeProvider.vue'
import { useLocaleSwitcher } from '@composables/useLocaleSwitcher'
import { APP_CONTEXT_KEY, useAppContext } from './appContext'
import type { Unsubscribe } from '@/events/types'

const context = useAppContext()
const themeStore = useThemeStore()
const layoutStore = useLayoutStore()
const dialogStore = useDialogStore()
const notificationStore = useNotificationStore()
const { localeStore } = useLocaleSwitcher()
const subscriptions: Unsubscribe[] = []

provide(APP_CONTEXT_KEY, context)

const naiveTheme = computed(() => {
  return themeStore.isDark ? darkTheme : null
})

const themeOverrides = {
  common: {
    primaryColor: '#5B8DEF',
    primaryColorHover: '#7BA4F4',
    primaryColorPressed: '#4A7AD4',
    primaryColorSuppl: '#7C6FF2',
    bodyColor: '#0D0D0F',
    cardColor: '#1A1A1E',
    modalColor: '#252529',
    popoverColor: '#252529',
    borderColor: '#2A2A30',
    dividerColor: '#1F1F25',
    inputColor: '#151519',
    textColor1: '#EDEDEF',
    textColor2: '#9D9DA3',
    textColor3: '#6B6B72',
    textColorDisabled: '#4A4A50',
    fontSize: '13px',
    fontSizeSmall: '12px',
    fontSizeMini: '11px',
    borderRadius: '8px',
    borderRadiusSmall: '6px',
    heightSmall: '28px',
    heightMedium: '32px',
    heightLarge: '36px',
  },
}

onMounted(() => {
  subscriptions.push(
    context.events.subscribe('command-palette:open', () => {
      layoutStore.openCommandPalette()
    }),
    context.events.subscribe('command-palette:close', () => {
      layoutStore.closeCommandPalette()
    }),
    context.events.subscribe('command-palette:toggle', () => {
      layoutStore.toggleCommandPalette()
    }),
    context.events.subscribe('sidebar:toggle', () => {
      layoutStore.toggleSidebar()
    }),
    context.events.subscribe('inspector:toggle', () => {
      layoutStore.toggleInspector()
    }),
    context.events.subscribe('global-search:toggle', () => {
      if (layoutStore.globalSearchOpen) {
        layoutStore.closeGlobalSearch()
        return
      }

      layoutStore.openGlobalSearch()
    }),
    context.events.subscribe('notification:show', ({ payload }) => {
      notificationStore.notify({
        type: payload.type,
        title: payload.title,
        content: payload.content,
        duration: payload.duration,
        closable: payload.closable,
      })
    }),
    context.events.subscribe('dialog:show', ({ payload }) => {
      void dialogStore.openDialog({
        type: payload.type,
        title: payload.title,
        content: payload.content,
        props: payload.props,
      })
    }),
  )
})

onUnmounted(() => {
  for (const unsubscribe of subscriptions.splice(0)) {
    unsubscribe()
  }
})
</script>
