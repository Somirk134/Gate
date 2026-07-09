import type { Router } from 'vue-router'
import type { AppContext } from './AppContext'
import { AppLifecyclePhase } from './lifecycle'
import { WINDOW_SERVICE } from '@/services/tokens'

export function installAppRouterGuards(router: Router, context: AppContext) {
  router.beforeEach(async (to) => {
    if (
      context.lifecycle.phase === AppLifecyclePhase.Starting ||
      context.lifecycle.phase === AppLifecyclePhase.Initializing
    ) {
      await context.events.publish('navigation:request', {
        path: to.fullPath,
        replace: true,
      })
    }

    return true
  })

  router.afterEach((to) => {
    const title =
      typeof to.meta.title === 'string'
        ? `${to.meta.title} - ${context.environment.name}`
        : context.environment.name

    context.services.resolve(WINDOW_SERVICE).setTitle(title)
  })
}
