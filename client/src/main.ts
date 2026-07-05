import { createApp } from "vue"
import { createPinia } from "pinia"
import { createRouter, createWebHistory } from "vue-router"
import { createI18n } from "vue-i18n"

import App from "./App.vue"
import routes from "./router"
import "./styles/tokens.css"
import "./styles/animations.css"
import { designSystemPlugin } from "./plugins/designSystem"

import en from "./locales/en"
import zhCN from "./locales/zh-CN"

const savedLocale = localStorage.getItem('locale')
const defaultLocale = savedLocale || 'zh-CN'

const app = createApp(App)

const router = createRouter({
    history: createWebHistory(),
    routes,
})

const i18n = createI18n({
    legacy: false,
    locale: defaultLocale,
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
app.use(designSystemPlugin)

app.mount("#app")
