import { createApp } from "vue"
import { createPinia } from "pinia"
import { createRouter, createWebHistory } from "vue-router"
import { createI18n } from "vue-i18n"

import App from "./App.vue"
import routes from "./router"
import "./styles/tokens.css"
import "./styles/animations.css"
import { AppBootstrap } from "./core"
import { designSystemPlugin } from "./plugins/designSystem"
import { APP_CONTEXT_KEY, setApplicationContext } from "./providers/appContext"

import en from "./locales/en"
import zhCN from "./locales/zh-CN"

const app = createApp(App)

const router = createRouter({
    history: createWebHistory(),
    routes,
})

const i18n = createI18n({
    legacy: false,
    locale: "zh-CN",
    fallbackLocale: "en",
    messages: {
        en,
        "zh-CN": zhCN,
    },
})

const pinia = createPinia()

const application = await AppBootstrap.create({
    app,
    router,
})

setApplicationContext(application.context)
app.provide(APP_CONTEXT_KEY, application.context)
i18n.global.locale.value =
    application.context.configuration.get<"zh-CN" | "en">("locale") ?? "zh-CN"

app.use(router)
app.use(i18n)
app.use(pinia)
app.use(designSystemPlugin)

app.mount("#app")
await application.run()
