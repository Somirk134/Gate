import { createApp } from "vue"
import { createPinia } from "pinia"
import { createRouter, createWebHistory } from "vue-router"
import { createI18n } from "vue-i18n"

import App from "./App.vue"
import routes from "./router"
import "./styles/tokens.css"

import en from "./locales/en"
import zhCN from "./locales/zh-CN"

const app = createApp(App)

const router = createRouter({
    history: createWebHistory(),
    routes,
})

const i18n = createI18n({
    legacy: false,
    locale: "en",
    fallbackLocale: "en",
    messages: {
        en,
        "zh-CN": zhCN,
    },
})

const pinia = createPinia()

app.use(router)
app.use(i18n)
app.use(pinia)

app.mount("#app")
