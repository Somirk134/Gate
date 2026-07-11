import type { App } from 'vue'
import type { Router } from 'vue-router'
import type { AppContext } from './AppContext'
import { registerCommandShortcuts } from './registerCommandShortcuts'
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

  registerCommandShortcuts(context)

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
