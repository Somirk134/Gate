import { watch } from "vue"
import { useLocaleSwitcher, type SupportedLocale } from "@composables/useLocaleSwitcher"
import { useSettingsStore } from "../stores"

export function useLocaleSetting() {
  const settingsStore = useSettingsStore()
  const { currentLocale, setLocale } = useLocaleSwitcher()

  if (settingsStore.read("general.language") !== currentLocale.value) {
    settingsStore.setValue("general.language", currentLocale.value)
  }

  const stopSettingToLocale = watch(
    () => settingsStore.read("general.language"),
    (value) => {
      if (value !== "zh-CN" && value !== "en") return
      if (currentLocale.value === value) return
      setLocale(value as SupportedLocale)
    },
    { immediate: true },
  )

  const stopLocaleToSetting = watch(currentLocale, (value) => {
    if (settingsStore.read("general.language") === value) return
    settingsStore.setValue("general.language", value)
  })

  return {
    stop: () => {
      stopSettingToLocale()
      stopLocaleToSetting()
    },
  }
}
