import { storeToRefs } from "pinia"
import { useSettingsStore } from "../stores"

export function useSettings() {
  const store = useSettingsStore()
  const refs = storeToRefs(store)

  return {
    ...refs,
    read: store.read,
    getValue: store.getValue,
    setValue: store.setValue,
    resetSetting: store.resetSetting,
    resetCategory: store.resetCategory,
    resetAll: store.resetAll,
    importSettings: store.importSettings,
    exportSettings: store.exportSettings,
    watchSetting: store.watchSetting,
    setActiveCategory: store.setActiveCategory,
    setActiveGroup: store.setActiveGroup,
    setSelectedSetting: store.setSelectedSetting,
    setSearchQuery: store.setSearchQuery,
    setCategoryFilter: store.setCategoryFilter,
    runMockAction: store.runMockAction,
  }
}
