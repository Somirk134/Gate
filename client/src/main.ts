import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { createRouter, createWebHistory } from 'vue-router'

import App from './App.vue'
import routes from './router'
import './styles/tokens.css'
import './styles/animations.css'
import { AppBootstrap } from './core'
import { designSystemPlugin } from './plugins/designSystem'
import { APP_CONTEXT_KEY, setApplicationContext } from './providers/appContext'
import { initAppearancePreferences } from './composables/useAppearancePreferences'
import { i18n, persistRuntimeLocale, resolveInitialLocale } from './i18n'

const app = createApp(App)

initAppearancePreferences()

const router = createRouter({
  history: createWebHistory(),
  routes,
})

const pinia = createPinia()

const application = await AppBootstrap.create({
  app,
  router,
})

setApplicationContext(application.context)
app.provide(APP_CONTEXT_KEY, application.context)
const configuredLocale = await resolveInitialLocale(application.context.configuration.get('locale'))
i18n.global.locale.value = configuredLocale
application.context.configuration.set('locale', configuredLocale)
void persistRuntimeLocale(configuredLocale)

app.use(router)
app.use(i18n)
app.use(pinia)
app.use(designSystemPlugin)

app.mount('#app')
await application.run()
