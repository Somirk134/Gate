import type {
  SettingCategory,
  SettingCategoryId,
  SettingContext,
  SettingItem,
  SettingValidation,
  SettingValue,
} from '../types'
import { i18n } from '@/i18n'

function t(key: string, params?: Record<string, unknown>): string {
  return (i18n.global as unknown as { t: (key: string, params?: Record<string, unknown>) => string }).t(
    key,
    params,
  )
}

export function flattenSettingItems(categories: SettingCategory[]) {
  return categories.flatMap((category) =>
    category.groups.flatMap((group) =>
      group.items.map((item) => ({
        category,
        group,
        item,
      })),
    ),
  )
}

export function getDefaultValues(categories: SettingCategory[]) {
  return Object.fromEntries(
    flattenSettingItems(categories).map(({ item }) => [
      item.key,
      cloneSettingValue(item.defaultValue),
    ]),
  ) as Record<string, SettingValue>
}

export function findSettingContext(
  categories: SettingCategory[],
  settingIdOrKey: string | null | undefined,
): SettingContext | null {
  if (!settingIdOrKey) return null

  for (const category of categories) {
    for (const group of category.groups) {
      const item = group.items.find(
        (candidate) => candidate.id === settingIdOrKey || candidate.key === settingIdOrKey,
      )
      if (item) return { category, group, item }
    }
  }

  return null
}

export function getCategoryById(categories: SettingCategory[], id: SettingCategoryId) {
  return categories.find((category) => category.id === id) ?? categories[0]
}

export function isEqualSettingValue(left: SettingValue, right: SettingValue) {
  if (Array.isArray(left) || Array.isArray(right)) {
    return JSON.stringify(left ?? null) === JSON.stringify(right ?? null)
  }

  return left === right
}

export function cloneSettingValue(value: SettingValue): SettingValue {
  if (Array.isArray(value)) return [...value]
  return value
}

export function validateSettingValue(item: SettingItem, value: SettingValue) {
  const validation = item.validation
  if (!validation) return undefined

  const requiredMessage = validateRequired(validation, value)
  if (requiredMessage) return requiredMessage

  if (typeof value === 'number') {
    if (typeof validation.min === 'number' && value < validation.min) {
      return validation.message ?? t('settings.validation.min', { min: validation.min })
    }

    if (typeof validation.max === 'number' && value > validation.max) {
      return validation.message ?? t('settings.validation.max', { max: validation.max })
    }
  }

  if (typeof value === 'string' && validation.pattern) {
    const pattern = new RegExp(validation.pattern)
    if (!pattern.test(value)) return validation.message ?? t('settings.validation.pattern')
  }

  return undefined
}

export function formatSettingValue(value: SettingValue) {
  if (Array.isArray(value)) return value.join(', ')
  if (value === null) return t('settings.value.none')
  if (typeof value === 'boolean') return value ? t('settings.value.enabled') : t('settings.value.disabled')
  return String(value)
}

function validateRequired(validation: SettingValidation, value: SettingValue) {
  if (!validation.required) return undefined
  if (value === null) return validation.message ?? t('settings.validation.required')
  if (typeof value === 'string' && value.trim().length === 0)
    return validation.message ?? t('settings.validation.required')
  if (Array.isArray(value) && value.length === 0)
    return validation.message ?? t('settings.validation.required')
  return undefined
}
