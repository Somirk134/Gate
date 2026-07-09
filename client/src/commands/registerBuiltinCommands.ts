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
    navigate(router, 'navigation.dashboard', '打开首页', 'Dashboard', '/', 'dashboard', [
      '首页',
      'home',
      'dashboard',
    ]),
    navigate(router, 'navigation.projects', '打开项目', 'Projects', '/projects', 'projects', [
      '项目',
      'project',
    ]),
    navigate(router, 'navigation.tunnels', '打开隧道', 'Tunnels', '/tunnels', 'router', [
      '隧道',
      'tunnel',
    ]),
    navigate(router, 'navigation.servers', '打开服务器', 'Servers', '/servers', 'servers', [
      '服务器',
      'server',
    ]),
    navigate(router, 'navigation.logs', '打开日志', 'Logs', '/logs', 'logs', ['日志', 'log']),
    navigate(router, 'navigation.settings', '打开设置', 'Settings', '/settings', 'settings', [
      '设置',
      'settings',
    ]),
    navigate(router, 'navigation.about', '打开关于', 'About', '/about', 'about', ['关于', 'about']),
    navigate(router, 'navigation.help', '打开帮助', 'Help', '/help', 'help', ['帮助', 'help']),
    {
      id: 'tunnel.create',
      title: '创建隧道',
      description: '打开隧道创建向导',
      category: 'tunnel',
      icon: 'plus',
      shortcut: 'Ctrl+N',
      keywords: ['隧道', '创建', 'create', 'tunnel', 'new'],
      handler: async () => {
        await router.push('/tunnels?create=1')
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
    reserved('project.create', '创建项目', 'project', 'projects'),
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

function navigate(
  router: Router,
  id: string,
  title: string,
  description: string,
  path: string,
  icon: string,
  keywords: string[],
): Command {
  return {
    id,
    title,
    description,
    category: 'navigation',
    icon,
    keywords,
    handler: async () => {
      await router.push(path)
    },
  }
}
