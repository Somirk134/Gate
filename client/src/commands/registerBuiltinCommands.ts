import type { Router } from 'vue-router'
import type { AppContext } from '@/core/AppContext'
import type { Command } from './types'

export function registerBuiltinCommands(context: AppContext, router: Router) {
  const commands: Command[] = [
    {
      id: 'app.commandPalette.open',
      title: 'Open command palette',
      titleKey: 'commands.openCommandPalette',
      category: 'application',
      categoryKey: 'commands.category.application',
      icon: 'search',
      keywords: ['command', 'palette', 'search'],
      handler: async ({ context: app }) => {
        await app.events.publish('command-palette:open', undefined)
      },
    },
    {
      id: 'app.commandPalette.toggle',
      title: 'Toggle command palette',
      titleKey: 'commands.toggleCommandPalette',
      category: 'application',
      categoryKey: 'commands.category.application',
      icon: 'search',
      shortcut: 'Ctrl+K',
      keywords: ['command', 'palette', 'search'],
      handler: async ({ context: app }) => {
        await app.events.publish('command-palette:toggle', undefined)
      },
    },
    {
      id: 'settings.open',
      title: 'Open Settings',
      titleKey: 'commands.openSettings',
      category: 'settings',
      categoryKey: 'commands.category.settings',
      icon: 'settings',
      shortcut: 'Ctrl+,',
      keywords: ['settings', 'preferences', 'configuration'],
      handler: async () => {
        await router.push('/settings')
      },
    },
    navigate(router, 'navigation.dashboard', 'commands.navigation.dashboard', '/', 'dashboard', [
      'home',
      'dashboard',
    ]),
    navigate(router, 'navigation.projects', 'commands.navigation.projects', '/projects', 'projects', [
      'project',
      'workspace',
    ]),
    navigate(router, 'navigation.tunnels', 'commands.navigation.tunnels', '/tunnels', 'router', [
      'tunnel',
      'runtime',
    ]),
    navigate(router, 'navigation.servers', 'commands.navigation.servers', '/servers', 'servers', [
      'server',
      'gateway',
    ]),
    navigate(router, 'navigation.logs', 'commands.navigation.logs', '/logs', 'logs', ['log']),
    navigate(router, 'navigation.settings', 'commands.navigation.settings', '/settings', 'settings', [
      'settings',
    ]),
    navigate(router, 'navigation.about', 'commands.navigation.about', '/about', 'about', ['about']),
    navigate(router, 'navigation.help', 'commands.navigation.help', '/help', 'help', ['help']),
    {
      id: 'tunnel.create',
      title: 'Create Tunnel',
      titleKey: 'commands.createTunnel',
      description: 'Open the tunnel creation wizard',
      descriptionKey: 'commands.createTunnelDescription',
      category: 'tunnel',
      categoryKey: 'commands.category.tunnel',
      icon: 'plus',
      shortcut: 'Ctrl+N',
      keywords: ['create', 'tunnel', 'new'],
      handler: async () => {
        await router.push('/tunnels?create=1')
      },
    },
    {
      id: 'app.sidebar.toggle',
      title: 'Toggle sidebar',
      titleKey: 'commands.toggleSidebar',
      category: 'application',
      categoryKey: 'commands.category.application',
      icon: 'sidebar',
      shortcut: 'Ctrl+\\',
      keywords: ['sidebar', 'navigation'],
      handler: async ({ context: app }) => {
        await app.events.publish('sidebar:toggle', undefined)
      },
    },
    {
      id: 'app.inspector.toggle',
      title: 'Toggle inspector',
      titleKey: 'commands.toggleInspector',
      category: 'application',
      categoryKey: 'commands.category.application',
      icon: 'inspector',
      shortcut: 'Ctrl+Shift+I',
      keywords: ['inspector', 'panel'],
      handler: async ({ context: app }) => {
        await app.events.publish('inspector:toggle', undefined)
      },
    },
    {
      id: 'app.globalSearch.toggle',
      title: 'Toggle global search',
      titleKey: 'commands.toggleGlobalSearch',
      category: 'application',
      categoryKey: 'commands.category.application',
      icon: 'search',
      shortcut: 'Ctrl+Shift+K',
      keywords: ['global', 'search'],
      handler: async ({ context: app }) => {
        await app.events.publish('global-search:toggle', undefined)
      },
    },
    {
      id: 'project.quickOpen',
      title: 'Quick open project',
      titleKey: 'commands.quickOpenProject',
      category: 'project',
      categoryKey: 'commands.category.project',
      icon: 'projects',
      shortcut: 'Ctrl+P',
      keywords: ['project', 'open', 'quick'],
      handler: async ({ context: app }) => {
        await app.events.publish('command:reserved', {
          id: 'project.quickOpen',
        })
      },
    },
    reserved('project.create', 'commands.reserved.createProject', 'project', 'projects'),
    reserved('project.delete', 'commands.reserved.deleteProject', 'project', 'trash'),
    reserved('tunnel.start', 'commands.reserved.startTunnel', 'tunnel', 'play'),
    reserved('tunnel.stop', 'commands.reserved.stopTunnel', 'tunnel', 'stop'),
    reserved('server.connect', 'commands.reserved.connectServer', 'server', 'servers'),
  ]

  for (const command of commands) {
    if (!context.commands.has(command.id)) {
      context.commands.register(command)
    }
  }
}

function reserved(id: string, titleKey: string, category: string, icon: string): Command {
  return {
    id,
    title: id,
    titleKey,
    category,
    categoryKey: `commands.category.${category}`,
    icon,
    handler: async ({ context, args }) => {
      await context.events.publish('command:reserved', { id, args })
    },
  }
}

function navigate(
  router: Router,
  id: string,
  titleKey: string,
  path: string,
  icon: string,
  keywords: string[],
): Command {
  return {
    id,
    title: id,
    titleKey,
    description: path,
    descriptionKey: titleKey.replace('commands.navigation.', 'commands.navigationDescription.'),
    category: 'navigation',
    categoryKey: 'commands.category.navigation',
    icon,
    keywords,
    handler: async () => {
      await router.push(path)
    },
  }
}
