import type { App } from 'vue'
import type { Router } from 'vue-router'
import type { AppContext } from './AppContext'
import { installAppRouterGuards } from './routerGuard'
import {
  ERROR_HANDLER_SERVICE,
  SHORTCUT_SERVICE,
  THEME_SERVICE,
  WINDOW_SERVICE,
} from '@/services/tokens'

export function registerApplicationInitializers(context: AppContext, app: App, router: Router) {
  context.services.resolve(ERROR_HANDLER_SERVICE).installVue(app)

  context.services.resolve(WINDOW_SERVICE).setTitle(context.environment.name)

  installAppRouterGuards(router, context)

  context.services.resolve(SHORTCUT_SERVICE).register({
    id: 'shortcut.commandPalette.toggle',
    commandId: 'app.commandPalette.toggle',
    shortcut: context.configuration.get<string>('shortcuts.app.commandPalette.toggle') ?? 'Ctrl+K',
  })

  context.services.resolve(SHORTCUT_SERVICE).register({
    id: 'shortcut.project.create',
    commandId: 'project.create',
    shortcut: context.configuration.get<string>('shortcuts.project.create') ?? 'Ctrl+N',
  })

  context.services.resolve(SHORTCUT_SERVICE).register({
    id: 'shortcut.project.quickOpen',
    commandId: 'project.quickOpen',
    shortcut: context.configuration.get<string>('shortcuts.project.quickOpen') ?? 'Ctrl+P',
  })

  context.services.resolve(SHORTCUT_SERVICE).register({
    id: 'shortcut.settings.open',
    commandId: 'settings.open',
    shortcut: context.configuration.get<string>('shortcuts.settings.open') ?? 'Ctrl+,',
  })

  context.services.resolve(SHORTCUT_SERVICE).register({
    id: 'shortcut.sidebar.toggle',
    commandId: 'app.sidebar.toggle',
    shortcut: 'Ctrl+\\',
  })

  context.services.resolve(SHORTCUT_SERVICE).register({
    id: 'shortcut.inspector.toggle',
    commandId: 'app.inspector.toggle',
    shortcut: 'Ctrl+Shift+I',
  })

  context.services.resolve(SHORTCUT_SERVICE).register({
    id: 'shortcut.globalSearch.toggle',
    commandId: 'app.globalSearch.toggle',
    shortcut: 'Ctrl+Shift+K',
  })

  context.initializers.register({
    id: 'theme.start',
    priority: 100,
    initialize: () => {
      context.services.resolve(THEME_SERVICE).start()
    },
  })

  context.initializers.register({
    id: 'errors.start',
    priority: 90,
    initialize: () => {
      context.services.resolve(ERROR_HANDLER_SERVICE).start()
    },
  })

  context.initializers.register({
    id: 'shortcuts.start',
    priority: 10,
    initialize: () => {
      context.services.resolve(SHORTCUT_SERVICE).start()
    },
  })
}
