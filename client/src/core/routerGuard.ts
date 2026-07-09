import type { Router } from 'vue-router'
import { watch } from 'vue'
import type { AppContext } from './AppContext'
import { AppLifecyclePhase } from './lifecycle'
import { WINDOW_SERVICE } from '@/services/tokens'
import { i18n } from '@/i18n'

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

  const updateWindowTitle = (to = router.currentRoute.value) => {
    const titleKey = typeof to.meta.titleKey === 'string' ? to.meta.titleKey : undefined
    const title = titleKey
      ? `${i18n.global.t(titleKey)} - ${context.environment.name}`
      : context.environment.name

    context.services.resolve(WINDOW_SERVICE).setTitle(title)
  }

  router.afterEach((to) => updateWindowTitle(to))
  watch(() => i18n.global.locale.value, () => updateWindowTitle())
}
