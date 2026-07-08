<template>
  <section class="settings-page beta-settings">
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
          <span>{{ localize(category.label) }}</span>
        </button>
      </aside>

      <main class="settings-panel">
        <section v-if="selectedCategory" class="settings-category-panel">
          <div class="settings-category-panel__heading">
            <span><GIcon :name="selectedCategory.icon" :size="20" /></span>
            <div>
              <p>{{ localize(selectedCategory.kicker) }}</p>
              <h2>{{ localize(selectedCategory.label) }}</h2>
              <small>{{ localize(selectedCategory.description) }}</small>
            </div>
          </div>

          <div class="settings-groups">
            <article
              v-for="(group, groupIndex) in selectedCategory.groups"
              :key="`${selectedCategory.id}-${groupIndex}`"
              class="settings-group-card">
              <header>
                <div>
                  <h3>{{ localize(group.title) }}</h3>
                  <p>{{ localize(group.description) }}</p>
                </div>
              </header>

              <div class="settings-items">
                <div
                  v-for="item in group.items"
                  :key="item.key"
                  class="settings-row"
                  :class="{ 'settings-row--action': item.control === 'action' }">
                  <div>
                    <strong>{{ localize(item.label) }}</strong>
                    <span>{{ localize(item.description) }}</span>
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
                      v-for="option in normalizeOptions(item.options)"
                      :key="String(option.value)"
                      :value="option.value">
                      {{ localize(option.label) }}
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
                    {{ localize(item.buttonText ?? '执行') }}
                  </GButton>

                  <code v-else>{{ values[item.key] }}</code>
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
import { TauriIpcClient } from '@/ipc'
import './styles/settings.css'

type SettingControl = 'switch' | 'select' | 'number' | 'readonly' | 'action'
type SettingAction =
  | 'restoreDefaults'
  | 'exportConfig'
  | 'importConfig'
  | 'backupConfig'
  | 'resetCache'
  | 'clearLogs'
  | 'openOnboarding'

interface SettingItem {
  key: string
  label: LocalizedText
  description: LocalizedText
  control: SettingControl
  options?: SettingOption[]
  value: string | number | boolean
  icon?: string
  buttonText?: LocalizedText
  action?: SettingAction
}

interface SettingGroup {
  title: LocalizedText
  description: LocalizedText
  items: SettingItem[]
}

interface SettingCategory {
  id: string
  label: LocalizedText
  kicker: LocalizedText
  description: LocalizedText
  icon: string
  groups: SettingGroup[]
}

type LocalizedText = string | { zh: string; en: string }
type SettingOption = string | { label: LocalizedText; value: string | number | boolean }

const L = (zh: string, en: string): LocalizedText => ({ zh, en })
const O = (zh: string, en: string, value: string | number | boolean): SettingOption => ({
  label: L(zh, en),
  value,
})

const query = ref('')
const activeCategory = ref('general')
const importInput = ref<HTMLInputElement | null>(null)
const { toast } = useFeedback()
const { t, locale } = useI18n()
const { currentLocale, setLocale } = useLocaleSwitcher()
const themeStore = useThemeStore()
const ipc = new TauriIpcClient()
let hydrating = false

const categories: SettingCategory[] = [
  {
    id: 'general',
    label: L('通用', 'General'),
    kicker: L('启动与工作区', 'Startup and workspace'),
    description: L(
      '控制应用启动、默认行为和日常使用偏好。',
      'Control startup, defaults, and daily preferences.',
    ),
    icon: 'settings',
    groups: [
      {
        title: L('启动', 'Startup'),
        description: L(
          '保持普通开发者第一次打开时足够清晰。',
          'Keep first launch clear for everyday developers.',
        ),
        items: [
          {
            key: 'language',
            label: { zh: '语言', en: 'Language' },
            description: {
              zh: '切换整个界面的显示语言，立即生效。',
              en: 'Change the interface language. Applies immediately.',
            },
            control: 'select',
            options: [O('简体中文', 'Simplified Chinese', 'zh-CN'), O('英文', 'English', 'en')],
            value: 'zh-CN',
          },
          {
            key: 'launchAtLogin',
            label: L('开机启动', 'Launch at login'),
            description: L(
              '登录系统后自动启动 Gate。',
              'Start Gate automatically after system login.',
            ),
            control: 'switch',
            value: false,
          },
          {
            key: 'showWelcome',
            label: L('显示欢迎页', 'Show welcome'),
            description: L(
              '首次启动时展示智能新手引导。',
              'Show the guided onboarding on first launch.',
            ),
            control: 'switch',
            value: true,
          },
          {
            key: 'openOnboarding',
            label: L('重新打开新手引导', 'Open onboarding again'),
            description: L(
              '从头开始聊天式配置流程，不会改动 Runtime。',
              'Restart the guided setup without changing the Runtime.',
            ),
            control: 'action',
            value: '',
            icon: 'sparkles',
            buttonText: L('打开', 'Open'),
            action: 'openOnboarding',
          },
        ],
      },
      {
        title: L('工作区', 'Workspace'),
        description: L(
          '决定默认进入位置和运行反馈。',
          'Choose the default page and interaction feedback.',
        ),
        items: [
          {
            key: 'defaultPage',
            label: L('默认页面', 'Default page'),
            description: L('应用启动后打开的页面。', 'Page opened after the app starts.'),
            control: 'select',
            options: [
              O('首页', 'Dashboard', 'Dashboard'),
              O('项目', 'Projects', 'Projects'),
              O('隧道', 'Tunnels', 'Tunnels'),
              O('日志', 'Logs', 'Logs'),
            ],
            value: 'Dashboard',
          },
          {
            key: 'confirmBeforeStop',
            label: L('停止前确认', 'Confirm before stopping'),
            description: L(
              '停止 Tunnel 或项目时使用统一确认对话框。',
              'Ask before stopping a tunnel or project.',
            ),
            control: 'switch',
            value: true,
          },
        ],
      },
    ],
  },
  {
    id: 'network',
    label: L('网络', 'Network'),
    kicker: L('连接能力', 'Connectivity'),
    description: L(
      '网络重试、超时和代理相关偏好。',
      'Network retry, timeout, and proxy preferences.',
    ),
    icon: 'network',
    groups: [
      {
        title: L('连接', 'Connection'),
        description: L(
          '只调整客户端体验，不改变通信协议。',
          'Only changes client behavior, not the communication protocol.',
        ),
        items: [
          {
            key: 'timeout',
            label: L('连接超时', 'Connection timeout'),
            description: L(
              '客户端等待服务器响应的时间。',
              'How long the client waits for server response.',
            ),
            control: 'number',
            value: 10,
          },
          {
            key: 'retry',
            label: L('自动重试', 'Auto retry'),
            description: L(
              '连接失败后自动重试。',
              'Retry automatically after connection failures.',
            ),
            control: 'switch',
            value: true,
          },
        ],
      },
    ],
  },
  {
    id: 'server',
    label: L('服务器', 'Server'),
    kicker: L('中继与运行时', 'Relay and runtime'),
    description: L(
      '服务器显示、健康检查和默认区域。',
      'Server display, health checks, and default region.',
    ),
    icon: 'servers',
    groups: [
      {
        title: L('默认服务器', 'Default Server'),
        description: L(
          '让创建 Tunnel 时默认选择更合理。',
          'Make tunnel creation defaults more convenient.',
        ),
        items: [
          {
            key: 'serverRegion',
            label: L('默认区域', 'Default region'),
            description: L(
              '创建时优先使用的服务器区域。',
              'Preferred server region during creation.',
            ),
            control: 'select',
            options: [
              O('自动', 'Auto', 'Auto'),
              O('东京节点', 'Tokyo Edge', 'Tokyo Edge'),
              O('新加坡节点', 'Singapore Hub', 'Singapore Hub'),
              O('法兰克福中继', 'Frankfurt Relay', 'Frankfurt Relay'),
            ],
            value: 'Auto',
          },
          {
            key: 'healthPolling',
            label: L('健康轮询', 'Health polling'),
            description: L('在首页展示服务器健康状态。', 'Show server health on the dashboard.'),
            control: 'switch',
            value: true,
          },
        ],
      },
    ],
  },
  {
    id: 'tunnel',
    label: L('隧道', 'Tunnel'),
    kicker: L('创建默认值', 'Creation defaults'),
    description: L('隧道创建向导和运行默认值。', 'Tunnel wizard and runtime defaults.'),
    icon: 'router',
    groups: [
      {
        title: L('向导默认值', 'Wizard Defaults'),
        description: L(
          '减少重复输入，保持创建流程不超过一分钟。',
          'Reduce repeated input and keep creation fast.',
        ),
        items: [
          {
            key: 'defaultProtocol',
            label: L('默认协议', 'Default protocol'),
            description: L(
              '创建向导默认选择的协议。',
              'Protocol selected by default in the wizard.',
            ),
            control: 'select',
            options: ['HTTP', 'TCP'],
            value: 'HTTP',
          },
          {
            key: 'autoStartTunnel',
            label: L('创建后自动启动', 'Auto-start after creation'),
            description: L(
              '创建成功后立即启动隧道。',
              'Start the tunnel immediately after creation.',
            ),
            control: 'switch',
            value: false,
          },
        ],
      },
    ],
  },
  {
    id: 'appearance',
    label: L('外观', 'Appearance'),
    kicker: L('视觉样式', 'Visual style'),
    description: L('主题、字体、密度和动效偏好。', 'Theme, font, density, and motion preferences.'),
    icon: 'palette',
    groups: [
      {
        title: L('界面', 'Interface'),
        description: L('统一现代桌面应用视觉体验。', 'Tune the desktop interface appearance.'),
        items: [
          {
            key: 'theme',
            label: L('主题', 'Theme'),
            description: L('选择深色、浅色或跟随系统。', 'Choose dark, light, or system theme.'),
            control: 'select',
            options: [
              O('深色', 'Dark', 'Dark'),
              O('浅色', 'Light', 'Light'),
              O('跟随系统', 'System', 'System'),
            ],
            value: 'Dark',
          },
          {
            key: 'fontSize',
            label: { zh: '字体大小', en: 'Font Size' },
            description: {
              zh: '调整全局基础字号，立即生效。',
              en: 'Adjust the global base font size. Applies immediately.',
            },
            control: 'select',
            options: [
              { label: { zh: '紧凑', en: 'Compact' }, value: 'compact' },
              { label: { zh: '舒适', en: 'Comfortable' }, value: 'comfortable' },
              { label: { zh: '较大', en: 'Large' }, value: 'large' },
              { label: { zh: '特大', en: 'Extra Large' }, value: 'extra-large' },
            ],
            value: 'comfortable',
          },
          {
            key: 'density',
            label: L('界面密度', 'Interface density'),
            description: L(
              '控制卡片、表格和侧边栏间距。',
              'Adjust spacing in cards, tables, and sidebars.',
            ),
            control: 'select',
            options: [O('舒适', 'Comfortable', 'Comfortable'), O('紧凑', 'Compact', 'Compact')],
            value: 'Comfortable',
          },
          {
            key: 'reduceMotion',
            label: L('减少动画', 'Reduce motion'),
            description: L(
              '降低页面、对话框和提示动效。',
              'Reduce page, dialog, and toast animations.',
            ),
            control: 'switch',
            value: false,
          },
        ],
      },
    ],
  },
  {
    id: 'advanced',
    label: L('高级', 'Advanced'),
    kicker: L('高级用户', 'Power user'),
    description: L(
      '日志、缓存、诊断和实验选项集中到高级区。',
      'Logs, cache, diagnostics, and advanced options.',
    ),
    icon: 'sliders',
    groups: [
      {
        title: L('诊断', 'Diagnostics'),
        description: L(
          '默认保持安静，只在需要时打开。',
          'Keep quiet by default and enable only when needed.',
        ),
        items: [
          {
            key: 'verboseLogs',
            label: L('详细日志', 'Verbose logs'),
            description: L(
              '输出更多调试信息到日志页面。',
              'Output more diagnostic details to Logs.',
            ),
            control: 'switch',
            value: false,
          },
          {
            key: 'logRetention',
            label: L('日志保留', 'Log retention'),
            description: L('本地最多保留的日志条数。', 'Maximum local log entries to keep.'),
            control: 'number',
            value: 100000,
          },
        ],
      },
    ],
  },
  {
    id: 'maintenance',
    label: L('维护', 'Maintenance'),
    kicker: L('配置与清理', 'Config and cleanup'),
    description: L(
      '恢复默认设置、导入导出配置、备份配置、重置缓存和清理日志。',
      'Restore defaults, import/export config, backup, reset cache, and clean logs.',
    ),
    icon: 'sliders',
    groups: [
      {
        title: L('配置', 'Configuration'),
        description: L(
          '所有配置操作都可视化，不需要手动编辑文件。',
          'Manage configuration visually without editing files by hand.',
        ),
        items: [
          {
            key: 'restoreDefaults',
            label: L('恢复默认设置', 'Restore defaults'),
            description: L(
              '将界面偏好恢复为推荐默认值。',
              'Restore interface preferences to recommended defaults.',
            ),
            control: 'action',
            value: '',
            icon: 'refresh',
            buttonText: L('恢复', 'Restore'),
            action: 'restoreDefaults',
          },
          {
            key: 'exportConfig',
            label: L('导出配置', 'Export config'),
            description: L('导出当前设置为 JSON 文件。', 'Export current settings as a JSON file.'),
            control: 'action',
            value: '',
            icon: 'download',
            buttonText: L('导出', 'Export'),
            action: 'exportConfig',
          },
          {
            key: 'importConfig',
            label: L('导入配置', 'Import config'),
            description: L('从 JSON 文件导入设置。', 'Import settings from a JSON file.'),
            control: 'action',
            value: '',
            icon: 'upload',
            buttonText: L('导入', 'Import'),
            action: 'importConfig',
          },
          {
            key: 'backupConfig',
            label: L('备份配置', 'Backup config'),
            description: L('生成带时间戳的配置备份文件。', 'Create a timestamped settings backup.'),
            control: 'action',
            value: '',
            icon: 'save',
            buttonText: L('备份', 'Backup'),
            action: 'backupConfig',
          },
        ],
      },
      {
        title: L('本地数据', 'Local Data'),
        description: L(
          '清理不会影响服务端，也不会新增或删除隧道协议。',
          'Cleanup does not affect the server or tunnel protocols.',
        ),
        items: [
          {
            key: 'resetCache',
            label: L('重置缓存', 'Reset cache'),
            description: L(
              '清理欢迎向导、最近服务器和连接历史缓存。',
              'Clear onboarding, recent servers, and connection history cache.',
            ),
            control: 'action',
            value: '',
            icon: 'trash',
            buttonText: L('重置', 'Reset'),
            action: 'resetCache',
          },
          {
            key: 'clearLogs',
            label: L('清理日志', 'Clear logs'),
            description: L(
              '清理本地日志视图缓存；服务端日志不受影响。',
              'Clear local log view cache; server logs are not affected.',
            ),
            control: 'action',
            value: '',
            icon: 'logs',
            buttonText: L('清理', 'Clear'),
            action: 'clearLogs',
          },
        ],
      },
    ],
  },
  {
    id: 'about',
    label: L('关于', 'About'),
    kicker: L('版本与许可', 'Version and legal'),
    description: L(
      '版本信息、更新状态和项目链接。',
      'Version information, update status, and project links.',
    ),
    icon: 'info-circle',
    groups: [
      {
        title: 'Gate',
        description: L('当前桌面客户端版本。', 'Current desktop client version.'),
        items: [
          {
            key: 'version',
            label: L('版本', 'Version'),
            description: L('当前安装的 Gate Desktop 版本。', 'Installed Gate Desktop version.'),
            control: 'readonly',
            value: '0.1.0',
          },
          {
            key: 'channel',
            label: L('通道', 'Channel'),
            description: L('当前更新通道。', 'Current update channel.'),
            control: 'readonly',
            value: 'Beta Sprint 1',
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
const defaultValues = Object.fromEntries(
  categories.flatMap((category) =>
    category.groups.flatMap((group) => group.items.map((item) => [item.key, item.value])),
  ),
) as Record<string, string | number | boolean>

onMounted(async () => {
  hydrating = true
  try {
    values.language = currentLocale.value
    values.theme = getCurrentThemeSetting()
    values.fontSize = getFontSizePreference()
    const saved = await ipc.invoke<Record<string, string>>('get_config')
    for (const key of Object.keys(values)) {
      if (saved[key] === undefined) continue
      values[key] = parseSettingValue(saved[key], values[key])
    }
    applyRuntimePreferences()
  } catch (err) {
    toast.error(err instanceof Error ? err.message : '设置加载失败')
  } finally {
    hydrating = false
  }
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
      localize(category.label),
      localize(category.kicker),
      localize(category.description),
      ...category.groups.flatMap((group) => [
        localize(group.title),
        localize(group.description),
        ...group.items.flatMap((item) => [localize(item.label), localize(item.description)]),
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

watch(visibleCategories, (list) => {
  if (!list.some((category) => category.id === activeCategory.value)) {
    activeCategory.value = list[0]?.id ?? 'general'
  }
})

function runSettingAction(action?: SettingAction) {
  if (!action) return
  if (action === 'restoreDefaults') {
    Object.assign(values, defaultValues)
    void persistAllSettings()
    toast.success('已恢复默认设置')
  }
  if (action === 'exportConfig') {
    downloadConfig('gate-config.json')
    toast.success('配置已导出')
  }
  if (action === 'backupConfig') {
    const date = new Date().toISOString().replace(/[:.]/g, '-')
    downloadConfig(`gate-config-backup-${date}.json`)
    toast.success('配置备份已生成')
  }
  if (action === 'importConfig') {
    importInput.value?.click()
  }
  if (action === 'resetCache') {
    localStorage.removeItem('gate.firstLaunch.completed')
    localStorage.removeItem('gate.smartOnboarding.completed')
    localStorage.removeItem('gate.smartOnboarding.neverShow')
    localStorage.removeItem('gate.smartOnboarding.draft')
    localStorage.removeItem('gate.recentServers')
    localStorage.removeItem('gate.connectionHistory')
    localStorage.removeItem('gate.welcome.dismissed')
    toast.success('缓存已重置，下次启动会重新显示向导')
  }
  if (action === 'clearLogs') {
    void clearRuntimeLogs()
  }
  if (action === 'openOnboarding') {
    window.dispatchEvent(new CustomEvent('gate:onboarding:open', { detail: { restart: true } }))
    toast.success('已打开新手引导')
  }
}

async function clearRuntimeLogs() {
  try {
    await ipc.invoke<void>('runtime_clear_logs')
    window.dispatchEvent(new CustomEvent('gate:logs:cleared'))
    toast.success('日志已清理')
  } catch (err) {
    toast.error(err instanceof Error ? err.message : '日志清理失败')
  }
}

function applyRuntimePreferences() {
  if (values.language === 'zh-CN' || values.language === 'en') {
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

function localize(text: LocalizedText) {
  if (typeof text === 'string') return text
  return locale.value === 'en' ? text.en : text.zh
}

function normalizeOptions(options: SettingOption[] = []) {
  return options.map((option) => {
    if (typeof option === 'string') return { label: option, value: option }
    return option
  })
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
      toast.success('配置已导入')
    } catch {
      toast.error('导入失败：配置文件不是有效 JSON')
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
    toast.error(err instanceof Error ? err.message : '设置保存失败')
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
</script>
