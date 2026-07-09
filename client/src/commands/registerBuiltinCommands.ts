import type { Router } from 'vue-router'
import type { AppContext } from '@/core/AppContext'
import type { Command } from './types'

export function registerBuiltinCommands(context: AppContext, router: Router) {
  const commands: Command[] = [
    {
      id: 'app.commandPalette.open',
      title: '打开命令面板',
      category: 'application',
      icon: 'search',
      keywords: ['command', 'palette', 'search'],
      handler: async ({ context: app }) => {
        await app.events.publish('command-palette:open', undefined)
      },
    },
    {
      id: 'app.commandPalette.toggle',
      title: '切换命令面板',
      category: 'application',
      icon: 'search',
      shortcut: 'Ctrl+K',
      keywords: ['command', 'palette', 'search'],
      handler: async ({ context: app }) => {
        await app.events.publish('command-palette:toggle', undefined)
      },
    },
    {
      id: 'settings.open',
      title: '打开设置',
      category: 'settings',
      icon: 'settings',
      shortcut: 'Ctrl+,',
      keywords: ['settings', 'preferences', 'configuration'],
      handler: async () => {
        await router.push('/settings')
      },
    },
    {
      id: 'app.sidebar.toggle',
      title: '切换侧边栏',
      category: 'application',
      icon: 'sidebar',
      shortcut: 'Ctrl+\\',
      keywords: ['sidebar', 'navigation'],
      handler: async ({ context: app }) => {
        await app.events.publish('sidebar:toggle', undefined)
      },
    },
    {
      id: 'app.inspector.toggle',
      title: '切换检查器',
      category: 'application',
      icon: 'inspector',
      shortcut: 'Ctrl+Shift+I',
      keywords: ['inspector', 'panel'],
      handler: async ({ context: app }) => {
        await app.events.publish('inspector:toggle', undefined)
      },
    },
    {
      id: 'app.globalSearch.toggle',
      title: '切换全局搜索',
      category: 'application',
      icon: 'search',
      shortcut: 'Ctrl+Shift+K',
      keywords: ['global', 'search'],
      handler: async ({ context: app }) => {
        await app.events.publish('global-search:toggle', undefined)
      },
    },
    {
      id: 'project.quickOpen',
      title: '快速打开项目',
      category: 'project',
      icon: 'projects',
      shortcut: 'Ctrl+P',
      keywords: ['project', 'open', 'quick'],
      handler: async ({ context: app }) => {
        await app.events.publish('command:reserved', {
          id: 'project.quickOpen',
        })
      },
    },
    reserved('project.create', '创建项目', 'project', 'projects', 'Ctrl+N'),
    reserved('project.delete', '删除项目', 'project', 'trash'),
    reserved('tunnel.start', '启动隧道', 'tunnel', 'play'),
    reserved('tunnel.stop', '停止隧道', 'tunnel', 'stop'),
    reserved('server.connect', '连接服务器', 'server', 'servers'),
  ]

  for (const command of commands) {
    if (!context.commands.has(command.id)) {
      context.commands.register(command)
    }
  }
}

function reserved(
  id: string,
  title: string,
  category: string,
  icon: string,
  shortcut?: string,
): Command {
  return {
    id,
    title,
    category,
    icon,
    shortcut,
    handler: async ({ context, args }) => {
      await context.events.publish('command:reserved', { id, args })
    },
  }
}
