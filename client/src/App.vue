<template>
    <n-config-provider :theme="theme" :locale="locale" :date-locale="dateLocale">
        <n-message-provider>
            <n-dialog-provider>
                <n-notification-provider>
                    <router-view />
                </n-notification-provider>
            </n-dialog-provider>
        </n-message-provider>
    </n-config-provider>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { useTheme } from '@hooks/useTheme'
import { useI18n } from 'vue-i18n'
import type { NLocale, NDateLocale } from 'naive-ui/es/locales'

const { theme } = useTheme()
const { locale: i18nLocale } = useI18n()

const locale = ref<NLocale | null>(null)
const dateLocale = ref<NDateLocale | null>(null)

const loadLocales = async () => {
    if (i18nLocale.value === 'zh-CN') {
        locale.value = (await import('naive-ui/es/locales/common/zhCN')).default
        dateLocale.value = (await import('naive-ui/es/locales/date/zhCN')).default
    } else {
        locale.value = (await import('naive-ui/es/locales/common/enUS')).default
        dateLocale.value = (await import('naive-ui/es/locales/date/enUS')).default
    }
}

onMounted(loadLocales)
</script>
