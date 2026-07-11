<template>
  <section class="settings-page rc-settings">
    <input
      ref="importInput"
      class="settings-import-input"
      type="file"
      accept="application/json,.json"
      @change="handleImportFile" />

    <header class="settings-hero">
      <div>
        <p>{{ t('settings.preferences') }}</p>
        <h1>{{ t('settings.title') }}</h1>
      </div>
      <label class="settings-hero__search">
        <GIcon name="search" :size="15" />
        <input v-model.trim="query" :placeholder="t('settings.searchPlaceholder')" />
      </label>
    </header>

    <div class="settings-layout">
      <aside class="settings-nav" :aria-label="t('settings.categories')">
        <button
          v-for="category in visibleCategories"
          :key="category.id"
          type="button"
          :class="{ active: activeCategory === category.id }"
          @click="activeCategory = category.id">
          <GIcon :name="category.icon" :size="16" />
          <span>{{ t(category.labelKey) }}</span>
        </button>
      </aside>

      <main class="settings-panel">
        <section v-if="selectedCategory" class="settings-category-panel">
          <div class="settings-category-panel__heading">
            <span><GIcon :name="selectedCategory.icon" :size="20" /></span>
            <div>
              <p>{{ t(selectedCategory.kickerKey) }}</p>
              <h2>{{ t(selectedCategory.labelKey) }}</h2>
              <small>{{ t(selectedCategory.descriptionKey) }}</small>
            </div>
          </div>

          <div class="settings-groups">
            <article
              v-for="group in selectedCategory.groups"
              :key="`${selectedCategory.id}-${group.titleKey}`"
              class="settings-group-card">
              <header>
                <div>
                  <h3>{{ t(group.titleKey) }}</h3>
                  <p>{{ t(group.descriptionKey) }}</p>
                </div>
              </header>

              <div class="settings-items">
                <div
                  v-for="item in group.items"
                  :key="item.key"
                  class="settings-row"
                  :class="{ 'settings-row--action': item.control === 'action' }">
                  <div>
                    <strong>{{ t(item.labelKey) }}</strong>
                    <span>{{ t(item.descriptionKey) }}</span>
                  </div>

                  <button
                    v-if="item.control === 'switch'"
                    type="button"
                    class="settings-switch"
                    :class="{ active: values[item.key] }"
                    :aria-pressed="Boolean(values[item.key])"
                    @click="values[item.key] = !values[item.key]">
                    <i />
                  </button>

                  <select v-else-if="item.control === 'select'" v-model="values[item.key]">
                    <option
                      v-for="option in item.options ?? []"
                      :key="String(option.value)"
                      :value="option.value">
                      {{ t(option.labelKey) }}
                    </option>
                  </select>

                  <input
                    v-else-if="item.control === 'number'"
                    v-model.number="values[item.key]"
                    type="number"
                    min="1" />

                  <GButton
                    v-else-if="item.control === 'action'"
                    variant="secondary"
                    :icon="item.icon"
                    @click="runSettingAction(item.action)">
                    {{ t(item.buttonKey ?? 'settings.actions.run') }}
                  </GButton>

                  <code v-else>{{ readonlyValue(item.key) }}</code>
                </div>
              </div>
            </article>
          </div>
        </section>

        <div v-else class="settings-empty">
          <GIcon name="search" :size="28" />
          <h2>{{ t('settings.noMatches') }}</h2>
          <p>{{ t('settings.noMatchesHint') }}</p>
        </div>
      </main>
    </div>

    <Transition name="settings-restore">
      <div v-if="restorePreview" class="settings-restore-backdrop" role="presentation">
        <article
          class="settings-restore-dialog"
          role="dialog"
          aria-modal="true"
          aria-labelledby="settings-restore-title">
          <header>
            <div>
              <p>{{ t('backup.restore.kicker') }}</p>
              <h2 id="settings-restore-title">{{ t('backup.restore.title') }}</h2>
            </div>
            <button
              type="button"
              :aria-label="t('common.cancel')"
              :disabled="restoreBusy"
              @click="closeRestorePreview">
              <GIcon name="close" :size="16" />
            </button>
          </header>

          <section class="settings-restore-summary">
            <div>
              <span>{{ t('backup.restore.backupVersion') }}</span>
              <strong>{{ restorePreview.version }} / v{{ restorePreview.appVersion }}</strong>
            </div>
            <div>
              <span>{{ t('backup.restore.createdAt') }}</span>
              <strong>{{ formatBackupDate(restorePreview.createdAt) }}</strong>
            </div>
            <div>
              <span>{{ t('backup.restore.entries') }}</span>
              <strong>{{ t('backup.restore.entryCount', { count: restorePreview.entries.length }) }}</strong>
            </div>
          </section>

          <section class="settings-restore-grid">
            <article v-for="row in restoreRows" :key="row.labelKey">
              <span>{{ t(row.labelKey) }}</span>
              <strong>{{ row.value }}</strong>
              <small>{{ t(row.descriptionKey) }}</small>
            </article>
          </section>

          <div class="settings-restore-warning">
            <GIcon name="alert-triangle" :size="18" />
            <div>
              <strong>{{ t('backup.restore.warningTitle') }}</strong>
              <p>{{ t('backup.restore.warningBody') }}</p>
            </div>
          </div>

          <p v-if="restoreError" class="settings-restore-error">{{ restoreError }}</p>

          <footer>
            <GButton variant="ghost" :disabled="restoreBusy" @click="closeRestorePreview">
              {{ t('common.cancel') }}
            </GButton>
            <GButton
              variant="primary"
              icon="upload"
              :loading="restoreBusy"
              @click="confirmRestoreBackup">
              {{ t('backup.restore.confirm') }}
            </GButton>
          </footer>
        </article>
      </div>
    </Transition>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import GButton from '@components/base/GButton.vue'
import GIcon from '@components/icons/GIcon.vue'
import {
  applyFontSizePreference,
  getFontSizePreference,
} from '@composables/useAppearancePreferences'
import { useFeedback } from '@composables/useFeedback'
import { useLocaleSwitcher } from '@composables/useLocaleSwitcher'
import { useThemeStore } from '@stores'
import { isTauri } from '@tauri-apps/api/core'
import { save } from '@tauri-apps/plugin-dialog'
import { TauriIpcClient } from '@/ipc'
import { backupService, type BackupPreview } from '@/services'
import { APP_RELEASE_CHANNEL, APP_VERSION } from '@/constants'
import { useProjectStore } from '@views/projects/store/project'
import { useServerStore } from '@views/servers'
import { useTunnelStore } from '@views/tunnels'

type SettingControl = 'switch' | 'select' | 'number' | 'readonly' | 'action'
type SettingAction =
  | 'restoreDefaults'
  | 'exportConfig'
  | 'importConfig'
  | 'backupConfig'
  | 'restoreBackup'
  | 'resetCache'
  | 'clearLogs'
  | 'openOnboarding'

interface SettingOption {
  labelKey: string
  value: string | number | boolean
}

interface SettingItem {
  key: string
  labelKey: string
  descriptionKey: string
  control: SettingControl
  options?: SettingOption[]
  value: string | number | boolean
  icon?: string
  buttonKey?: string
  action?: SettingAction
}

interface SettingGroup {
  titleKey: string
  descriptionKey: string
  items: SettingItem[]
}

interface SettingCategory {
  id: string
  labelKey: string
  kickerKey: string
  descriptionKey: string
  icon: string
  groups: SettingGroup[]
}

const option = (
  labelKey: string,
  value: string | number | boolean,
): SettingOption => ({ labelKey, value })

const query = ref('')
const activeCategory = ref('general')
const importInput = ref<HTMLInputElement | null>(null)
const { toast, notify } = useFeedback()
const { t, locale } = useI18n()
const { currentLocale, setLocale } = useLocaleSwitcher()
const themeStore = useThemeStore()
const projectStore = useProjectStore()
const serverStore = useServerStore()
const tunnelStore = useTunnelStore()
const ipc = new TauriIpcClient()
const restorePreview = ref<BackupPreview | null>(null)
const restoreBusy = ref(false)
const restoreError = ref('')
let hydrating = false

const categories: SettingCategory[] = [
  {
    id: 'general',
    labelKey: 'settings.general',
    kickerKey: 'settings.polish.general.kicker',
    descriptionKey: 'settings.polish.general.description',
    icon: 'settings',
    groups: [
      {
        titleKey: 'settings.polish.language.title',
        descriptionKey: 'settings.polish.language.description',
        items: [
          {
            key: 'language',
            labelKey: 'settings.language',
            descriptionKey: 'settings.desc.language',
            control: 'select',
            options: [
              option('settings.languageZh', 'zh-CN'),
              option('settings.languageEn', 'en-US'),
            ],
            value: 'zh-CN',
          },
          {
            key: 'openOnboarding',
            labelKey: 'settings.polish.openOnboarding.label',
            descriptionKey: 'settings.polish.openOnboarding.description',
            control: 'action',
            value: '',
            icon: 'sparkles',
            buttonKey: 'settings.polish.openOnboarding.button',
            action: 'openOnboarding',
          },
        ],
      },
      {
        titleKey: 'settings.polish.startup.title',
        descriptionKey: 'settings.polish.startup.description',
        items: [
          {
            key: 'launchAtLogin',
            labelKey: 'settings.startAtLogin',
            descriptionKey: 'settings.desc.startAtLogin',
            control: 'switch',
            value: false,
          },
          {
            key: 'showWelcome',
            labelKey: 'settings.polish.showWelcome.label',
            descriptionKey: 'settings.polish.showWelcome.description',
            control: 'switch',
            value: true,
          },
        ],
      },
    ],
  },
  {
    id: 'appearance',
    labelKey: 'settings.appearance',
    kickerKey: 'settings.polish.appearance.kicker',
    descriptionKey: 'settings.polish.appearance.description',
    icon: 'palette',
    groups: [
      {
        titleKey: 'settings.polish.interface.title',
        descriptionKey: 'settings.polish.interface.description',
        items: [
          {
            key: 'theme',
            labelKey: 'settings.theme',
            descriptionKey: 'settings.desc.theme',
            control: 'select',
            options: [
              option('settings.themeDark', 'Dark'),
              option('settings.themeLight', 'Light'),
              option('settings.themeSystem', 'System'),
            ],
            value: 'Dark',
          },
          {
            key: 'fontSize',
            labelKey: 'settings.fontSize',
            descriptionKey: 'settings.polish.fontSize.description',
            control: 'select',
            options: [
              option('settings.fontSizeCompact', 'compact'),
              option('settings.fontSizeComfortable', 'comfortable'),
              option('settings.fontSizeLarge', 'large'),
              option('settings.fontSizeExtraLarge', 'extra-large'),
            ],
            value: 'comfortable',
          },
          {
            key: 'reduceMotion',
            labelKey: 'settings.polish.reduceMotion.label',
            descriptionKey: 'settings.polish.reduceMotion.description',
            control: 'switch',
            value: false,
          },
        ],
      },
    ],
  },
  {
    id: 'network',
    labelKey: 'settings.network',
    kickerKey: 'settings.polish.network.kicker',
    descriptionKey: 'settings.polish.network.description',
    icon: 'network',
    groups: [
      {
        titleKey: 'settings.polish.connection.title',
        descriptionKey: 'settings.polish.connection.description',
        items: [
          {
            key: 'timeout',
            labelKey: 'settings.timeout',
            descriptionKey: 'settings.desc.timeout',
            control: 'number',
            value: 10,
          },
          {
            key: 'retry',
            labelKey: 'settings.polish.retry.label',
            descriptionKey: 'settings.polish.retry.description',
            control: 'switch',
            value: true,
          },
        ],
      },
    ],
  },
  {
    id: 'data',
    labelKey: 'settings.dataManagement.title',
    kickerKey: 'settings.dataManagement.kicker',
    descriptionKey: 'settings.dataManagement.description',
    icon: 'database',
    groups: [
      {
        titleKey: 'settings.dataManagement.backupGroup',
        descriptionKey: 'settings.dataManagement.backupDescription',
        items: [
          {
            key: 'backupConfig',
            labelKey: 'settings.dataManagement.exportBackup',
            descriptionKey: 'settings.dataManagement.exportBackupDesc',
            control: 'action',
            value: '',
            icon: 'download',
            buttonKey: 'settings.dataManagement.export',
            action: 'backupConfig',
          },
          {
            key: 'restoreBackup',
            labelKey: 'settings.dataManagement.restoreBackup',
            descriptionKey: 'settings.dataManagement.restoreBackupDesc',
            control: 'action',
            value: '',
            icon: 'upload',
            buttonKey: 'settings.dataManagement.restore',
            action: 'restoreBackup',
          },
          {
            key: 'exportConfig',
            labelKey: 'settings.dataManagement.exportConfig',
            descriptionKey: 'settings.dataManagement.exportConfigDesc',
            control: 'action',
            value: '',
            icon: 'save',
            buttonKey: 'settings.dataManagement.export',
            action: 'exportConfig',
          },
          {
            key: 'importConfig',
            labelKey: 'settings.dataManagement.importConfig',
            descriptionKey: 'settings.dataManagement.importConfigDesc',
            control: 'action',
            value: '',
            icon: 'upload',
            buttonKey: 'settings.dataManagement.import',
            action: 'importConfig',
          },
        ],
      },
      {
        titleKey: 'settings.dataManagement.localDataGroup',
        descriptionKey: 'settings.dataManagement.localDataDescription',
        items: [
          {
            key: 'resetCache',
            labelKey: 'settings.dataManagement.resetCache',
            descriptionKey: 'settings.dataManagement.resetCacheDesc',
            control: 'action',
            value: '',
            icon: 'trash',
            buttonKey: 'settings.dataManagement.reset',
            action: 'resetCache',
          },
          {
            key: 'clearLogs',
            labelKey: 'settings.dataManagement.clearLogs',
            descriptionKey: 'settings.dataManagement.clearLogsDesc',
            control: 'action',
            value: '',
            icon: 'logs',
            buttonKey: 'settings.dataManagement.clear',
            action: 'clearLogs',
          },
        ],
      },
    ],
  },
  {
    id: 'about',
    labelKey: 'nav.about',
    kickerKey: 'settings.polish.about.kicker',
    descriptionKey: 'settings.polish.about.description',
    icon: 'info-circle',
    groups: [
      {
        titleKey: 'common.appName',
        descriptionKey: 'settings.polish.about.groupDescription',
        items: [
          {
            key: 'version',
            labelKey: 'about.versionLabel',
            descriptionKey: 'settings.polish.about.versionDescription',
            control: 'readonly',
            value: APP_VERSION,
          },
          {
            key: 'channel',
            labelKey: 'about.channelLabel',
            descriptionKey: 'settings.polish.about.channelDescription',
            control: 'readonly',
            value: APP_RELEASE_CHANNEL,
          },
        ],
      },
    ],
  },
]

const values = reactive<Record<string, string | number | boolean>>(
  Object.fromEntries(
    categories.flatMap((category) =>
      category.groups.flatMap((group) => group.items.map((item) => [item.key, item.value])),
    ),
  ),
)
const defaultValues = { ...values }

onMounted(async () => {
  await hydrateSettingsFromRuntime()
})

watch(
  values,
  () => {
    applyRuntimePreferences()
    if (!hydrating) void persistAllSettings()
  },
  { deep: true },
)

const visibleCategories = computed(() => {
  const keyword = query.value.toLowerCase()
  if (!keyword) return categories
  return categories.filter((category) =>
    [
      t(category.labelKey),
      t(category.kickerKey),
      t(category.descriptionKey),
      ...category.groups.flatMap((group) => [
        t(group.titleKey),
        t(group.descriptionKey),
        ...group.items.flatMap((item) => [t(item.labelKey), t(item.descriptionKey)]),
      ]),
    ]
      .join(' ')
      .toLowerCase()
      .includes(keyword),
  )
})

const selectedCategory = computed(
  () =>
    visibleCategories.value.find((category) => category.id === activeCategory.value) ??
    visibleCategories.value[0],
)

const restoreRows = computed(() => {
  const contents = restorePreview.value?.contents
  if (!contents) return []
  return [
    {
      labelKey: 'backup.contents.projects',
      value: contents.projects,
      descriptionKey: 'backup.contents.projectsDesc',
    },
    {
      labelKey: 'backup.contents.servers',
      value: contents.servers,
      descriptionKey: 'backup.contents.serversDesc',
    },
    {
      labelKey: 'backup.contents.tunnels',
      value: contents.tunnels,
      descriptionKey: 'backup.contents.tunnelsDesc',
    },
    {
      labelKey: 'backup.contents.domains',
      value: contents.domains,
      descriptionKey: 'backup.contents.domainsDesc',
    },
    {
      labelKey: 'backup.contents.certificates',
      value: contents.certificates,
      descriptionKey: 'backup.contents.certificatesDesc',
    },
    {
      labelKey: 'backup.contents.settings',
      value: contents.settings,
      descriptionKey: 'backup.contents.settingsDesc',
    },
  ]
})

watch(visibleCategories, (list) => {
  if (!list.some((category) => category.id === activeCategory.value)) {
    activeCategory.value = list[0]?.id ?? 'general'
  }
})

async function hydrateSettingsFromRuntime() {
  hydrating = true
  try {
    Object.assign(values, defaultValues)
    values.language = currentLocale.value
    values.theme = getCurrentThemeSetting()
    values.fontSize = getFontSizePreference()
    const saved = await ipc.invoke<Record<string, string>>('get_config')
    for (const key of Object.keys(values)) {
      const raw = key === 'language' ? saved[key] ?? saved['app.locale'] : saved[key]
      if (raw === undefined) continue
      values[key] = parseSettingValue(raw, values[key])
    }
    if (values.language === 'en') values.language = 'en-US'
    applyRuntimePreferences()
  } catch (err) {
    toast.error(errorMessage(err, t('settings.notifications.loadFailed')))
  } finally {
    hydrating = false
  }
}

function runSettingAction(action?: SettingAction) {
  if (!action) return
  if (action === 'restoreDefaults') {
    Object.assign(values, defaultValues)
    void persistAllSettings()
    toast.success(t('settings.notifications.defaultsRestored'))
  }
  if (action === 'exportConfig') void exportConfig()
  if (action === 'backupConfig') void createGateBackup()
  if (action === 'restoreBackup') void chooseBackupForRestore()
  if (action === 'importConfig') importInput.value?.click()
  if (action === 'resetCache') resetLocalCache()
  if (action === 'clearLogs') void clearRuntimeLogs()
  if (action === 'openOnboarding') {
    window.dispatchEvent(new CustomEvent('gate:onboarding:open', { detail: { restart: true } }))
    toast.success(t('settings.notifications.onboardingOpened'))
  }
}

async function createGateBackup() {
  try {
    const destination = await backupService.chooseExportPath()
    if (!destination) return
    const result = await backupService.export(destination)
    notify.success(t('backup.export.success'), result.path)
  } catch (err) {
    toast.error(errorMessage(err, t('backup.export.failed')))
  }
}

async function exportConfig() {
  const filename = 'gate-config.json'
  const content = JSON.stringify(values, null, 2)

  if (!isTauri()) {
    downloadConfig(filename)
    notify.success(t('settings.notifications.configExported'), filename)
    return
  }

  try {
    const destination = await save({
      defaultPath: filename,
      filters: [{ name: 'Gate JSON', extensions: ['json'] }],
    })
    if (!destination) return

    const exportedPath = await ipc.invoke<string>('export_config_file', {
      path: destination,
      content,
    })
    notify.success(t('settings.notifications.configExported'), exportedPath)
  } catch (err) {
    toast.error(errorMessage(err, t('settings.notifications.configExportFailed')))
  }
}

async function chooseBackupForRestore() {
  try {
    const path = await backupService.chooseRestorePath()
    if (!path) return
    restorePreview.value = await backupService.preview(path)
    restoreError.value = ''
  } catch (err) {
    toast.error(errorMessage(err, t('backup.restore.previewFailed')))
  }
}

function closeRestorePreview() {
  if (restoreBusy.value) return
  restorePreview.value = null
  restoreError.value = ''
}

async function confirmRestoreBackup() {
  if (!restorePreview.value) return
  restoreBusy.value = true
  restoreError.value = ''
  try {
    const result = await backupService.restore(restorePreview.value.path)
    window.dispatchEvent(new CustomEvent('gate:backup:restored', { detail: result }))
    await refreshProductStateAfterRestore()
    toast.success(t(result.messageKey))
    restorePreview.value = null
  } catch (err) {
    restoreError.value = errorMessage(err, t('backup.restore.failed'))
    toast.error(t('backup.restore.failed'))
  } finally {
    restoreBusy.value = false
  }
}

async function refreshProductStateAfterRestore() {
  // 恢复会替换本地数据文件，刷新 Store 可避免界面继续显示旧配置。
  await Promise.all([
    projectStore.load(),
    serverStore.load(),
    tunnelStore.load(),
    hydrateSettingsFromRuntime(),
  ])
}

async function clearRuntimeLogs() {
  try {
    await ipc.invoke<void>('runtime_clear_logs')
    window.dispatchEvent(new CustomEvent('gate:logs:cleared'))
    toast.success(t('settings.notifications.logsCleared'))
  } catch (err) {
    toast.error(errorMessage(err, t('settings.notifications.logsClearFailed')))
  }
}

function resetLocalCache() {
  localStorage.removeItem('gate.firstLaunch.completed')
  localStorage.removeItem('gate.smartOnboarding.completed')
  localStorage.removeItem('gate.smartOnboarding.neverShow')
  localStorage.removeItem('gate.smartOnboarding.draft')
  localStorage.removeItem('gate.recentServers')
  localStorage.removeItem('gate.connectionHistory')
  localStorage.removeItem('gate.welcome.dismissed')
  toast.success(t('settings.notifications.cacheReset'))
}

function applyRuntimePreferences() {
  if (values.language === 'zh-CN' || values.language === 'en-US') {
    setLocale(values.language)
  }

  if (typeof values.fontSize === 'string') {
    applyFontSizePreference(values.fontSize)
  }

  const theme = String(values.theme).toLowerCase()
  if (theme === 'light') {
    themeStore.setTheme('light')
  } else if (theme === 'system') {
    themeStore.setTheme('auto')
  } else {
    themeStore.setTheme('dark')
  }
}

function getCurrentThemeSetting() {
  if (themeStore.currentMode === 'light') return 'Light'
  if (themeStore.currentMode === 'auto') return 'System'
  return 'Dark'
}

function readonlyValue(key: string) {
  return values[key]
}

function formatBackupDate(value: string) {
  const date = new Date(value)
  if (Number.isNaN(date.getTime())) return value
  return new Intl.DateTimeFormat(locale.value === 'en-US' ? 'en-US' : 'zh-CN', {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(date)
}

function downloadConfig(filename: string) {
  const blob = new Blob([JSON.stringify(values, null, 2)], { type: 'application/json' })
  const url = URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = filename
  link.click()
  URL.revokeObjectURL(url)
}

function handleImportFile(event: Event) {
  const input = event.target as HTMLInputElement
  const file = input.files?.[0]
  if (!file) return
  const reader = new FileReader()
  reader.onload = () => {
    try {
      const imported = JSON.parse(String(reader.result)) as Record<
        string,
        string | number | boolean
      >
      for (const [key, value] of Object.entries(imported)) {
        if (key in values && ['string', 'number', 'boolean'].includes(typeof value)) {
          values[key] = value
        }
      }
      void persistAllSettings()
      toast.success(t('settings.notifications.configImported'))
    } catch {
      toast.error(t('settings.notifications.configImportFailed'))
    } finally {
      input.value = ''
    }
  }
  reader.readAsText(file)
}

async function persistAllSettings() {
  try {
    await Promise.all(
      Object.entries(values).map(([key, value]) =>
        ipc.invoke<void>('set_config', {
          key,
          value: JSON.stringify(value),
        }),
      ),
    )
  } catch (err) {
    toast.error(errorMessage(err, t('settings.notifications.saveFailed')))
  }
}

function parseSettingValue(raw: string, fallback: string | number | boolean) {
  try {
    const parsed = JSON.parse(raw) as unknown
    if (typeof parsed === typeof fallback) return parsed as string | number | boolean
  } catch {
    if (typeof fallback === 'string') return raw
  }
  return fallback
}

function errorMessage(err: unknown, fallback: string) {
  if (err instanceof Error && err.message) return err.message
  if (typeof err === 'string') return err
  return fallback
}
</script>

<style src="./styles/settings.css"></style>
