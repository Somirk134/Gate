import { computed } from "vue"
import { storeToRefs } from "pinia"
import { useSettingsStore } from "../stores"
import type { SettingItem } from "../types"

export function useSettingReset() {
  const store = useSettingsStore()
  const { dirtyKeys, modifiedCount } = storeToRefs(store)

  function isModified(item: SettingItem) {
    return dirtyKeys.value.includes(item.key)
  }

  const hasModifiedSettings = computed(() => modifiedCount.value > 0)

  return {
    dirtyKeys,
    modifiedCount,
    hasModifiedSettings,
    isModified,
    resetSetting: store.resetSetting,
    resetCategory: store.resetCategory,
    resetAll: store.resetAll,
  }
}
