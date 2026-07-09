import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { zhCN, dateZhCN, enUS, dateEnUS } from 'naive-ui'
import type { NLocale, NDateLocale } from 'naive-ui/es/locales'
import { tryGetApplicationContext } from '@/providers/appContext'

export type SupportedLocale = 'zh-CN' | 'en'

export const locales = [
  { value: 'zh-CN' as SupportedLocale, label: '简体中文' },
  { value: 'en' as SupportedLocale, label: '英文' },
]

export function useLocaleSwitcher() {
  const { locale } = useI18n()

  const currentLocale = computed<SupportedLocale>(() => (locale.value === 'en' ? 'en' : 'zh-CN'))

  const localeStore = computed(() => ({
    naiveLocale: currentLocale.value === 'zh-CN' ? zhCN : (enUS as NLocale),
    naiveDateLocale: currentLocale.value === 'zh-CN' ? dateZhCN : (dateEnUS as NDateLocale),
  }))

  function setLocale(newLocale: SupportedLocale) {
    locale.value = newLocale
    tryGetApplicationContext()?.configuration.set('locale', newLocale)
  }

  return {
    locale,
    locales,
    currentLocale,
    localeStore,
    setLocale,
  }
}
