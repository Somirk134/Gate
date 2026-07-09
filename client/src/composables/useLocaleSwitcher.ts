import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { zhCN, dateZhCN, enUS, dateEnUS } from 'naive-ui'
import type { NLocale, NDateLocale } from 'naive-ui/es/locales'
import { tryGetApplicationContext } from '@/providers/appContext'
import { normalizeLocale, persistRuntimeLocale, type SupportedLocale } from '@/i18n'

export type { SupportedLocale } from '@/i18n'

export const locales = [
  { value: 'zh-CN' as SupportedLocale, labelKey: 'settings.languageZh' },
  { value: 'en-US' as SupportedLocale, labelKey: 'settings.languageEn' },
]

export function useLocaleSwitcher() {
  const { locale } = useI18n()

  const currentLocale = computed<SupportedLocale>(() => normalizeLocale(locale.value))

  const localeStore = computed(() => ({
    naiveLocale: currentLocale.value === 'zh-CN' ? zhCN : (enUS as NLocale),
    naiveDateLocale: currentLocale.value === 'zh-CN' ? dateZhCN : (dateEnUS as NDateLocale),
  }))

  function setLocale(newLocale: SupportedLocale) {
    const nextLocale = normalizeLocale(newLocale)
    locale.value = nextLocale
    tryGetApplicationContext()?.configuration.set('locale', nextLocale)
    void persistRuntimeLocale(nextLocale)
  }

  return {
    locale: currentLocale,
    locales,
    currentLocale,
    localeStore,
    setLocale,
  }
}
