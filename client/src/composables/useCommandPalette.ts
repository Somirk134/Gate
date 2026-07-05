import { computed, ref } from "vue"
import { useLayoutStore } from "@stores"
import { useAppContext } from "@/providers/appContext"
import type { CommandItem } from "@/types/shell"

export function useCommandPalette() {
  const context = useAppContext()
  const layoutStore = useLayoutStore()

  const query = ref("")
  const selectedIndex = ref(0)
  const recentCommands = ref<string[]>([])

  const isOpen = computed(() => layoutStore.commandPaletteOpen)

  const allCommands = computed<CommandItem[]>(() => {
    return context.commands.list().map((command) => ({
      id: command.id,
      title: command.title,
      subtitle: command.description,
      icon: command.icon,
      shortcut: command.shortcut,
      category: command.category,
      action: async () => {
        await context.commands.execute(command.id, { source: "command-palette" })
      },
      keywords: command.keywords,
    }))
  })

  const filteredCommands = computed<CommandItem[]>(() => {
    const keyword = query.value.trim().toLowerCase()

    if (!keyword) {
      return allCommands.value
    }

    return allCommands.value.filter((command) => {
      const haystack = [
        command.title,
        command.subtitle,
        command.category,
        ...(command.keywords ?? []),
      ].join(" ").toLowerCase()

      return haystack.includes(keyword)
    })
  })

  function open() {
    query.value = ""
    selectedIndex.value = 0
    layoutStore.openCommandPalette()
  }

  function close() {
    layoutStore.closeCommandPalette()
  }

  function selectNext() {
    if (filteredCommands.value.length === 0) {
      return
    }

    selectedIndex.value = (selectedIndex.value + 1) % filteredCommands.value.length
  }

  function selectPrev() {
    if (filteredCommands.value.length === 0) {
      return
    }

    selectedIndex.value =
      (selectedIndex.value - 1 + filteredCommands.value.length) % filteredCommands.value.length
  }

  function executeSelected() {
    const command = filteredCommands.value[selectedIndex.value]

    if (command) {
      void command.action()
      recordRecent(command.id)
      close()
    }
  }

  function executeCommand(id: string) {
    const command = allCommands.value.find((item) => item.id === id)

    if (command) {
      void command.action()
      recordRecent(id)
      close()
    }
  }

  function recordRecent(id: string) {
    recentCommands.value = [id, ...recentCommands.value.filter((item) => item !== id)].slice(0, 10)
  }

  return {
    query,
    selectedIndex,
    isOpen,
    allCommands,
    filteredCommands,
    open,
    close,
    selectNext,
    selectPrev,
    executeSelected,
    executeCommand,
  }
}
