import { computed, onMounted, onUnmounted } from "vue"
import { useSettingsStore } from "../stores"

export function useShortcut(options: { onSearch?: () => void } = {}) {
  const store = useSettingsStore()

  const shortcuts = computed(() =>
    store.allItems.filter((item) => item.control.type === "shortcut"),
  )

  function handleKeydown(event: KeyboardEvent) {
    const key = event.key.toLowerCase()
    const command = event.ctrlKey || event.metaKey

    if (command && key === "f") {
      event.preventDefault()
      options.onSearch?.()
    }
  }

  onMounted(() => window.addEventListener("keydown", handleKeydown))
  onUnmounted(() => window.removeEventListener("keydown", handleKeydown))

  return {
    shortcuts,
  }
}
