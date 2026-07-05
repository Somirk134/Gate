<template>
  <div class="settings-page">
    <h1 class="page-title">{{ t('settings.title') }}</h1>

    <div class="settings-layout">
      <nav class="settings-nav">
        <button v-for="section in sections" :key="section.id" class="settings-nav-item" :class="{ active: activeSection === section.id }" @click="activeSection = section.id">
          {{ section.label }}
        </button>
      </nav>

      <div class="settings-content">
        <!-- General -->
        <section v-if="activeSection === 'general'" class="settings-section">
          <h2 class="section-title">{{ t('settings.general') }}</h2>
          <div class="setting-item">
            <div class="setting-info"><span class="setting-label">{{ t('settings.language') }}</span><span class="setting-desc">{{ t('settings.desc.language') }}</span></div>
            <select v-model="locale" class="select-input">
              <option v-for="l in locales" :key="l.value" :value="l.value">{{ l.label }}</option>
            </select>
          </div>
          <div class="setting-item">
            <div class="setting-info"><span class="setting-label">{{ t('settings.startAtLogin') }}</span><span class="setting-desc">{{ t('settings.desc.startAtLogin') }}</span></div>
            <label class="toggle"><input type="checkbox" checked /><span class="toggle-slider"></span></label>
          </div>
          <div class="setting-item">
            <div class="setting-info"><span class="setting-label">{{ t('settings.minimizeToTray') }}</span><span class="setting-desc">{{ t('settings.desc.minimizeToTray') }}</span></div>
            <label class="toggle"><input type="checkbox" /><span class="toggle-slider"></span></label>
          </div>
        </section>

        <section v-if="activeSection === 'appearance'" class="settings-section">
          <h2 class="section-title">{{ t('settings.appearance') }}</h2>
          <div class="setting-item">
            <div class="setting-info"><span class="setting-label">{{ t('settings.theme') }}</span><span class="setting-desc">{{ t('settings.desc.theme') }}</span></div>
            <select class="select-input"><option>{{ t('settings.themeDark') }}</option><option>{{ t('settings.themeLight') }}</option></select>
          </div>
          <div class="setting-item">
            <div class="setting-info"><span class="setting-label">{{ t('settings.fontSize') }}</span><span class="setting-desc">{{ t('settings.desc.fontSize') }}</span></div>
            <select class="select-input"><option>12px</option><option selected>13px</option><option>14px</option></select>
          </div>
          <div class="setting-item">
            <div class="setting-info"><span class="setting-label">{{ t('settings.accentColor') }}</span><span class="setting-desc">{{ t('settings.desc.accentColor') }}</span></div>
            <div class="color-options">
              <span class="color-swatch" style="background:#5B8DEF" :class="{ active: true }"></span>
              <span class="color-swatch" style="background:#7C6FF2"></span>
              <span class="color-swatch" style="background:#22C55E"></span>
              <span class="color-swatch" style="background:#F59E0B"></span>
              <span class="color-swatch" style="background:#EF4444"></span>
            </div>
          </div>
        </section>

        <section v-if="activeSection === 'network'" class="settings-section">
          <h2 class="section-title">{{ t('settings.network') }}</h2>
          <div class="setting-item">
            <div class="setting-info"><span class="setting-label">{{ t('settings.proxy') }}</span><span class="setting-desc">{{ t('settings.desc.proxy') }}</span></div>
            <select class="select-input"><option>{{ t('settings.proxyNone') }}</option><option>{{ t('settings.proxySystem') }}</option><option>{{ t('settings.proxyCustom') }}</option></select>
          </div>
          <div class="setting-item">
            <div class="setting-info"><span class="setting-label">{{ t('settings.timeout') }}</span><span class="setting-desc">{{ t('settings.desc.timeout') }}</span></div>
            <select class="select-input"><option>15s</option><option selected>30s</option><option>60s</option></select>
          </div>
        </section>

        <section v-if="activeSection === 'updates'" class="settings-section">
          <h2 class="section-title">{{ t('settings.updates') }}</h2>
          <div class="setting-item">
            <div class="setting-info"><span class="setting-label">{{ t('settings.autoUpdate') }}</span><span class="setting-desc">{{ t('settings.desc.autoUpdate') }}</span></div>
            <label class="toggle"><input type="checkbox" checked /><span class="toggle-slider"></span></label>
          </div>
          <div class="setting-item">
            <div class="setting-info"><span class="setting-label">{{ t('settings.updateChannel') }}</span><span class="setting-desc">{{ t('settings.desc.updateChannel') }}</span></div>
            <select class="select-input"><option selected>{{ t('settings.channelStable') }}</option><option>{{ t('settings.channelBeta') }}</option></select>
          </div>
        </section>

        <section v-if="activeSection === 'developer'" class="settings-section">
          <h2 class="section-title">{{ t('settings.developer') }}</h2>
          <div class="setting-item">
            <div class="setting-info"><span class="setting-label">{{ t('settings.debugMode') }}</span><span class="setting-desc">{{ t('settings.desc.debugMode') }}</span></div>
            <label class="toggle"><input type="checkbox" /><span class="toggle-slider"></span></label>
          </div>
          <div class="setting-item">
            <div class="setting-info"><span class="setting-label">{{ t('settings.logLevel') }}</span><span class="setting-desc">{{ t('settings.desc.logLevel') }}</span></div>
            <select class="select-input"><option>{{ t('settings.logLevelDebug') }}</option><option selected>{{ t('settings.logLevelInfo') }}</option><option>{{ t('settings.logLevelWarning') }}</option><option>{{ t('settings.logLevelError') }}</option></select>
          </div>
        </section>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, watch } from "vue"
import { useI18n } from "vue-i18n"
import { useLocaleSwitcher } from "@composables/useLocaleSwitcher"
import { useConfiguration } from "@composables/useConfiguration"

const { t } = useI18n()
const { locale, locales } = useLocaleSwitcher()
const configuration = useConfiguration()

watch(locale, (val) => {
    configuration.set('locale', val)
})

const activeSection = ref("general")

const sections = reactive([
  { id: "general", label: t("settings.general") },
  { id: "appearance", label: t("settings.appearance") },
  { id: "network", label: t("settings.network") },
  { id: "updates", label: t("settings.updates") },
  { id: "developer", label: t("settings.developer") },
])
</script>

<style scoped>
.settings-page { max-width: 860px; margin: 0 auto; }
.page-title { font-size: var(--text-xl); font-weight: 600; margin-bottom: var(--space-6); }
.settings-layout { display: flex; gap: var(--space-6); }
.settings-nav { width: 160px; flex-shrink: 0; display: flex; flex-direction: column; gap: 2px; }
.settings-nav-item { display: block; width: 100%; text-align: left; padding: var(--space-2) var(--space-3); border: none; background: transparent; color: var(--text-secondary); font-family: var(--font-ui); font-size: var(--text-base); border-radius: var(--radius-md); cursor: pointer; transition: all var(--duration-micro) var(--ease-out); }
.settings-nav-item:hover { background: var(--bg-surface-hover); color: var(--text-primary); }
.settings-nav-item.active { background: var(--accent-primary-muted); color: var(--accent-primary); font-weight: 500; }
.settings-content { flex: 1; min-width: 0; }
.settings-section { margin-bottom: var(--space-8); }
.section-title { font-size: var(--text-lg); font-weight: 600; color: var(--text-primary); margin-bottom: var(--space-4); padding-bottom: var(--space-3); border-bottom: 1px solid var(--border-subtle); }
.setting-item { display: flex; align-items: center; justify-content: space-between; padding: var(--space-3) 0; }
.setting-item + .setting-item { border-top: 1px solid var(--border-subtle); }
.setting-label { display: block; font-size: var(--text-base); font-weight: 500; color: var(--text-primary); }
.setting-desc { display: block; font-size: var(--text-sm); color: var(--text-muted); margin-top: 2px; }
.select-input { height: 32px; padding: 0 var(--space-3); background: var(--bg-input); border: 1px solid var(--border-default); border-radius: var(--radius-md); color: var(--text-primary); font-family: var(--font-ui); font-size: var(--text-base); cursor: pointer; min-width: 140px; }
.toggle { position: relative; display: inline-block; width: 36px; height: 20px; cursor: pointer; }
.toggle input { opacity: 0; width: 0; height: 0; }
.toggle-slider { position: absolute; inset: 0; background: var(--border-strong); border-radius: var(--radius-full); transition: background var(--duration-standard) var(--ease-out); }
.toggle-slider::before { content: ""; position: absolute; width: 14px; height: 14px; left: 3px; bottom: 3px; background: white; border-radius: var(--radius-full); transition: transform var(--duration-standard) var(--ease-out); }
.toggle input:checked + .toggle-slider { background: var(--accent-primary); }
.toggle input:checked + .toggle-slider::before { transform: translateX(16px); }
.color-options { display: flex; gap: var(--space-2); }
.color-swatch { width: 24px; height: 24px; border-radius: var(--radius-full); cursor: pointer; border: 2px solid transparent; transition: all var(--duration-micro) var(--ease-out); }
.color-swatch:hover { transform: scale(1.15); }
.color-swatch.active { border-color: var(--text-primary); box-shadow: 0 0 0 2px var(--bg-app), 0 0 0 4px currentColor; }
</style>
