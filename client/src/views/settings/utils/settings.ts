import type {
  SettingCategory,
  SettingCategoryId,
  SettingContext,
  SettingItem,
  SettingValidation,
  SettingValue,
} from "../types"

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
    flattenSettingItems(categories).map(({ item }) => [item.key, cloneSettingValue(item.defaultValue)]),
  ) as Record<string, SettingValue>
}

export function findSettingContext(
  categories: SettingCategory[],
  settingIdOrKey: string | null | undefined,
): SettingContext | null {
  if (!settingIdOrKey) return null

  for (const category of categories) {
    for (const group of category.groups) {
      const item = group.items.find((candidate) => candidate.id === settingIdOrKey || candidate.key === settingIdOrKey)
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

  if (typeof value === "number") {
    if (typeof validation.min === "number" && value < validation.min) {
      return validation.message ?? `最小值为 ${validation.min}。`
    }

    if (typeof validation.max === "number" && value > validation.max) {
      return validation.message ?? `最大值为 ${validation.max}。`
    }
  }

  if (typeof value === "string" && validation.pattern) {
    const pattern = new RegExp(validation.pattern)
    if (!pattern.test(value)) return validation.message ?? "格式不正确。"
  }

  return undefined
}

export function formatSettingValue(value: SettingValue) {
  if (Array.isArray(value)) return value.join(", ")
  if (value === null) return "无"
  if (typeof value === "boolean") return value ? "已启用" : "已禁用"
  return String(value)
}

function validateRequired(validation: SettingValidation, value: SettingValue) {
  if (!validation.required) return undefined
  if (value === null) return validation.message ?? "此项为必填。"
  if (typeof value === "string" && value.trim().length === 0) return validation.message ?? "此项为必填。"
  if (Array.isArray(value) && value.length === 0) return validation.message ?? "此项为必填。"
  return undefined
}
