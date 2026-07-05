import { ref, computed } from 'vue'
import { useLayoutStore } from '@stores'
import type { CommandItem } from '@/types/shell'

/* ==================================================================
   useCommandPalette — 命令面板逻辑
   支持：导航、搜索、切换、执行命令
   快捷键：Ctrl + K
   ================================================================== */

export function useCommandPalette() {
  const layoutStore = useLayoutStore()

  const query = ref('')
  const selectedIndex = ref(0)
  const recentCommands = ref<string[]>([])

  const isOpen = computed(() => layoutStore.commandPaletteOpen)

  // 内置导航命令（后续可扩展为动态注册）
  const navigationCommands = computed<CommandItem[]>(() => [
    {
      id: 'nav-dashboard',
      title: 'Dashboard',
      subtitle: 'Navigate to Dashboard',
      icon: 'dashboard',
      category: 'navigation',
      action: () => navigate('/'),
      keywords: ['home', 'main', 'dashboard'],
    },
    {
      id: 'nav-projects',
      title: 'Projects',
      subtitle: 'Navigate to Projects',
      icon: 'projects',
      category: 'navigation',
      action: () => navigate('/projects'),
      keywords: ['project', 'list'],
    },
    {
      id: 'nav-servers',
      title: 'Servers',
      subtitle: 'Navigate to Servers',
      icon: 'servers',
      category: 'navigation',
      action: () => navigate('/servers'),
      keywords: ['server', 'host'],
    },
    {
      id: 'nav-logs',
      title: 'Logs',
      subtitle: 'Navigate to Logs',
      icon: 'logs',
      category: 'navigation',
      action: () => navigate('/logs'),
      keywords: ['log', 'history'],
    },
    {
      id: 'nav-settings',
      title: 'Settings',
      subtitle: 'Navigate to Settings',
      icon: 'settings',
      category: 'navigation',
      action: () => navigate('/settings'),
      keywords: ['config', 'preference'],
    },
  ])

  // 动作命令
  const actionCommands = computed<CommandItem[]>(() => [
    {
      id: 'action-toggle-sidebar',
      title: 'Toggle Sidebar',
      subtitle: 'Show or hide the sidebar',
      icon: 'sidebar',
      category: 'action',
      shortcut: 'Ctrl + \\',
      action: () => layoutStore.toggleSidebar(),
      keywords: ['sidebar', 'nav', 'hide', 'show'],
    },
    {
      id: 'action-toggle-inspector',
      title: 'Toggle Inspector',
      subtitle: 'Show or hide the right panel',
      icon: 'inspector',
      category: 'action',
      shortcut: 'Ctrl + Shift + I',
      action: () => layoutStore.toggleInspector(),
      keywords: ['inspector', 'panel', 'detail'],
    },
    {
      id: 'action-toggle-theme',
      title: 'Toggle Theme',
      subtitle: 'Switch between dark and light theme',
      icon: 'theme',
      category: 'settings',
      action: () => { /* theme toggle handled in theme store */ },
      keywords: ['theme', 'dark', 'light', 'color'],
    },
  ])

  const allCommands = computed<CommandItem[]>(() => [
    ...navigationCommands.value,
    ...actionCommands.value,
  ])

  const filteredCommands = computed<CommandItem[]>(() => {
    const q = query.value.trim().toLowerCase()
    if (!q) return allCommands.value
    return allCommands.value.filter(cmd => {
      const haystack = [
        cmd.title,
        cmd.subtitle,
        cmd.category,
        ...(cmd.keywords || []),
      ].join(' ').toLowerCase()
      return haystack.includes(q)
    })
  })

  function navigate(_path: string) {
    close()
    // 使用 window.location 或 router，这里预留接口
  }

  function open() {
    query.value = ''
    selectedIndex.value = 0
    layoutStore.openCommandPalette()
  }

  function close() {
    layoutStore.closeCommandPalette()
  }

  function selectNext() {
    if (filteredCommands.value.length === 0) return
    selectedIndex.value = (selectedIndex.value + 1) % filteredCommands.value.length
  }

  function selectPrev() {
    if (filteredCommands.value.length === 0) return
    selectedIndex.value = (selectedIndex.value - 1 + filteredCommands.value.length) % filteredCommands.value.length
  }

  function executeSelected() {
    const cmd = filteredCommands.value[selectedIndex.value]
    if (cmd) {
      cmd.action()
      recordRecent(cmd.id)
      close()
    }
  }

  function executeCommand(id: string) {
    const cmd = allCommands.value.find(c => c.id === id)
    if (cmd) {
      cmd.action()
      recordRecent(id)
      close()
    }
  }

  function recordRecent(id: string) {
    recentCommands.value = [id, ...recentCommands.value.filter(r => r !== id)].slice(0, 10)
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
