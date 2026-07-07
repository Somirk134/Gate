export type SettingCategoryId =
  | "general"
  | "appearance"
  | "workspace"
  | "project"
  | "tunnel"
  | "server"
  | "logs"
  | "network"
  | "https"
  | "notification"
  | "shortcut"
  | "storage"
  | "security"
  | "update"
  | "experimental"
  | "developer"
  | "about"

export type SettingControlType =
  | "switch"
  | "select"
  | "input"
  | "slider"
  | "number"
  | "checkbox"
  | "radio"
  | "color"
  | "folder"
  | "readonly"
  | "shortcut"
  | "action"

export type SettingPrimitive = string | number | boolean
export type SettingValue = SettingPrimitive | string[] | null

export interface SettingOption {
  label: string
  value: SettingPrimitive
  description?: string
  icon?: string
  badge?: string
  disabled?: boolean
}

export interface SettingAction {
  id: string
  label: string
  icon?: string
  variant?: "primary" | "secondary" | "ghost" | "danger"
  disabled?: boolean
  reserved?: boolean
}

export interface SettingValidation {
  required?: boolean
  min?: number
  max?: number
  step?: number
  pattern?: string
  message?: string
}

export type SettingControl =
  | { type: "switch"; disabled?: boolean }
  | { type: "select"; options: SettingOption[]; disabled?: boolean }
  | { type: "input"; placeholder?: string; inputType?: "text" | "password" | "url"; disabled?: boolean }
  | { type: "slider"; min: number; max: number; step?: number; unit?: string; disabled?: boolean }
  | { type: "number"; min?: number; max?: number; step?: number; unit?: string; disabled?: boolean }
  | { type: "checkbox"; disabled?: boolean }
  | { type: "radio"; options: SettingOption[]; disabled?: boolean }
  | { type: "color"; swatches?: string[]; disabled?: boolean; reserved?: boolean }
  | { type: "folder"; placeholder?: string; disabled?: boolean; reserved?: boolean }
  | { type: "readonly"; variant?: "text" | "mono" | "block" | "badge" }
  | { type: "shortcut"; editable?: boolean; disabled?: boolean }
  | { type: "action"; actions: SettingAction[] }

export interface SettingItem {
  id: string
  key: string
  categoryId: SettingCategoryId
  groupId: string
  label: string
  description: string
  control: SettingControl
  defaultValue: SettingValue
  recommendedValue?: SettingValue
  validation?: SettingValidation
  restartRequired?: boolean
  helpUrl?: string
  tags?: string[]
  status?: "stable" | "reserved" | "unimplemented"
  readonly?: boolean
}

export interface SettingGroup {
  id: string
  categoryId: SettingCategoryId
  label: string
  description?: string
  items: SettingItem[]
}

export interface SettingCategory {
  id: SettingCategoryId
  label: string
  description: string
  icon: string
  groups: SettingGroup[]
  order: number
}

export interface SettingContext {
  category: SettingCategory
  group: SettingGroup
  item: SettingItem
}

export interface SettingSearchMatch {
  field: "label" | "description" | "key" | "tags" | "category" | "group"
  score: number
}

export interface SettingSearchResult {
  category: SettingCategory
  group: SettingGroup
  item: SettingItem
  score: number
  matches: SettingSearchMatch[]
}

export interface SettingState {
  values: Record<string, SettingValue>
  defaults: Record<string, SettingValue>
  validationErrors: Record<string, string | undefined>
  activeCategoryId: SettingCategoryId
  activeGroupId: string | null
  selectedSettingId: string | null
  searchQuery: string
  categoryFilter: SettingCategoryId | "all"
}

export type SettingActionStatus = "idle" | "running" | "done"
