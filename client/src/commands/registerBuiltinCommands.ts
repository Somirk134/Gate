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
    navigate(router, 'navigation.domains', 'commands.navigation.domains', '/domains', 'globe', [
      'domain',
      'hostname',
      'dns',
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
  ]

  for (const command of commands) {
    if (!context.commands.has(command.id)) {
      context.commands.register(command)
    }
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
