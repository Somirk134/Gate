import { createI18n } from 'vue-i18n'
import { invoke } from '@tauri-apps/api/core'
import { createAtSafeMessageResolver } from '@/utils/i18n'
import zhCN from './locales/zh-CN'
import enUS from './locales/en-US'

export type SupportedLocale = 'zh-CN' | 'en-US'

export const DEFAULT_LOCALE: SupportedLocale = 'zh-CN'
export const FALLBACK_LOCALE: SupportedLocale = 'en-US'
export const LOCALE_CONFIG_KEY = 'app.locale'

export const i18n = createI18n({
  legacy: false,
  locale: DEFAULT_LOCALE,
  fallbackLocale: FALLBACK_LOCALE,
  messageResolver: createAtSafeMessageResolver(),
  messages: {
    'zh-CN': zhCN,
    'en-US': enUS,
  },
})

export function parseLocale(value: unknown): SupportedLocale | undefined {
  if (value === 'zh-CN') return 'zh-CN'
  if (value === 'en' || value === 'en-US') return 'en-US'
  return undefined
}

export function normalizeLocale(value: unknown): SupportedLocale {
  return parseLocale(value) ?? DEFAULT_LOCALE
}

export async function resolveInitialLocale(localValue: unknown): Promise<SupportedLocale> {
  const localLocale = parseLocale(localValue)

  try {
    const runtimeConfig = await invoke<Record<string, string>>('get_config')
    const runtimeLocale = parseLocale(
      runtimeConfig?.[LOCALE_CONFIG_KEY] ?? runtimeConfig?.language ?? runtimeConfig?.locale,
    )
    return runtimeLocale ?? localLocale ?? DEFAULT_LOCALE
  } catch {
    return localLocale ?? DEFAULT_LOCALE
  }
}

export async function persistRuntimeLocale(locale: SupportedLocale): Promise<void> {
  try {
    await invoke('set_config', {
      key: LOCALE_CONFIG_KEY,
      value: locale,
    })
  } catch {
    // 预览模式下没有 Tauri IPC，语言仍会写入前端配置。
  }
}
