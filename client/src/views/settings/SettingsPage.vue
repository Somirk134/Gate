<template>
  <section class="settings-page beta-settings">
    <input
      ref="importInput"
      class="settings-import-input"
      type="file"
      accept="application/json,.json"
      @change="handleImportFile"
    />

    <header class="settings-hero">
      <div>
        <p>Preferences</p>
        <h1>Settings</h1>
      </div>
      <label class="settings-hero__search">
        <GIcon name="search" :size="15" />
        <input v-model.trim="query" placeholder="搜索设置" />
      </label>
    </header>

    <div class="settings-layout">
      <aside class="settings-nav" aria-label="Settings categories">
        <button
          v-for="category in visibleCategories"
          :key="category.id"
          type="button"
          :class="{ active: activeCategory === category.id }"
          @click="activeCategory = category.id"
        >
          <GIcon :name="category.icon" :size="16" />
          <span>{{ category.label }}</span>
        </button>
      </aside>

      <main class="settings-panel">
        <section v-if="selectedCategory" class="settings-category-panel">
          <div class="settings-category-panel__heading">
            <span><GIcon :name="selectedCategory.icon" :size="20" /></span>
            <div>
              <p>{{ selectedCategory.kicker }}</p>
              <h2>{{ selectedCategory.label }}</h2>
              <small>{{ selectedCategory.description }}</small>
            </div>
          </div>

          <div class="settings-groups">
            <article v-for="group in selectedCategory.groups" :key="group.title" class="settings-group-card">
              <header>
                <div>
                  <h3>{{ group.title }}</h3>
                  <p>{{ group.description }}</p>
                </div>
              </header>

              <div class="settings-items">
                <div
                  v-for="item in group.items"
                  :key="item.key"
                  class="settings-row"
                  :class="{ 'settings-row--action': item.control === 'action' }"
                >
                  <div>
                    <strong>{{ item.label }}</strong>
                    <span>{{ item.description }}</span>
                  </div>

                  <button
                    v-if="item.control === 'switch'"
                    type="button"
                    class="settings-switch"
                    :class="{ active: values[item.key] }"
                    :aria-pressed="Boolean(values[item.key])"
                    @click="values[item.key] = !values[item.key]"
                  >
                    <i />
                  </button>

                  <select v-else-if="item.control === 'select'" v-model="values[item.key]">
                    <option v-for="option in item.options" :key="option" :value="option">{{ option }}</option>
                  </select>

                  <input
                    v-else-if="item.control === 'number'"
                    v-model.number="values[item.key]"
                    type="number"
                    min="1"
                  />

                  <GButton
                    v-else-if="item.control === 'action'"
                    variant="secondary"
                    :icon="item.icon"
                    @click="runSettingAction(item.action)"
                  >
                    {{ item.buttonText ?? "执行" }}
                  </GButton>

                  <code v-else>{{ values[item.key] }}</code>
                </div>
              </div>
            </article>
          </div>
        </section>

        <div v-else class="settings-empty">
          <GIcon name="search" :size="28" />
          <h2>没有匹配的设置</h2>
          <p>尝试搜索 General、Network、Tunnel 或 About。</p>
        </div>
      </main>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, reactive, ref, watch } from "vue"
import GButton from "@components/base/GButton.vue"
import GIcon from "@components/icons/GIcon.vue"
import { useFeedback } from "@composables/useFeedback"
import "./styles/settings.css"

type SettingControl = "switch" | "select" | "number" | "readonly" | "action"
type SettingAction =
  | "restoreDefaults"
  | "exportConfig"
  | "importConfig"
  | "backupConfig"
  | "resetCache"
  | "clearLogs"

interface SettingItem {
  key: string
  label: string
  description: string
  control: SettingControl
  options?: string[]
  value: string | number | boolean
  icon?: string
  buttonText?: string
  action?: SettingAction
}

interface SettingGroup {
  title: string
  description: string
  items: SettingItem[]
}

interface SettingCategory {
  id: string
  label: string
  kicker: string
  description: string
  icon: string
  groups: SettingGroup[]
}

const query = ref("")
const activeCategory = ref("general")
const importInput = ref<HTMLInputElement | null>(null)
const { toast } = useFeedback()

const categories: SettingCategory[] = [
  {
    id: "general",
    label: "General",
    kicker: "Startup and workspace",
    description: "控制应用启动、默认行为和日常使用偏好。",
    icon: "settings",
    groups: [
      {
        title: "Startup",
        description: "保持普通开发者第一次打开时足够清晰。",
        items: [
          { key: "launchAtLogin", label: "开机启动", description: "登录系统后自动启动 Gate。", control: "switch", value: false },
          { key: "showWelcome", label: "显示欢迎页", description: "首次启动时展示三步配置引导。", control: "switch", value: true },
        ],
      },
      {
        title: "Workspace",
        description: "决定默认进入位置和运行反馈。",
        items: [
          { key: "defaultPage", label: "默认页面", description: "应用启动后打开的页面。", control: "select", options: ["Dashboard", "Projects", "Tunnels", "Logs"], value: "Dashboard" },
          { key: "confirmBeforeStop", label: "停止前确认", description: "停止 Tunnel 或项目时使用统一确认对话框。", control: "switch", value: true },
        ],
      },
    ],
  },
  {
    id: "network",
    label: "Network",
    kicker: "Connectivity",
    description: "网络重试、超时和代理相关偏好。",
    icon: "network",
    groups: [
      {
        title: "Connection",
        description: "只调整客户端体验，不改变通信协议。",
        items: [
          { key: "timeout", label: "连接超时", description: "客户端等待服务器响应的时间。", control: "number", value: 10 },
          { key: "retry", label: "自动重试", description: "连接失败后自动重试。", control: "switch", value: true },
        ],
      },
    ],
  },
  {
    id: "server",
    label: "Server",
    kicker: "Relay and runtime",
    description: "服务器显示、健康检查和默认区域。",
    icon: "servers",
    groups: [
      {
        title: "Default Server",
        description: "让创建 Tunnel 时默认选择更合理。",
        items: [
          { key: "serverRegion", label: "默认区域", description: "创建时优先使用的服务器区域。", control: "select", options: ["Auto", "Tokyo Edge", "Singapore Hub", "Frankfurt Relay"], value: "Auto" },
          { key: "healthPolling", label: "健康轮询", description: "在 Dashboard 展示服务器健康状态。", control: "switch", value: true },
        ],
      },
    ],
  },
  {
    id: "tunnel",
    label: "Tunnel",
    kicker: "Creation defaults",
    description: "Tunnel 创建向导和运行默认值。",
    icon: "router",
    groups: [
      {
        title: "Wizard Defaults",
        description: "减少重复输入，保持创建流程不超过一分钟。",
        items: [
          { key: "defaultProtocol", label: "默认协议", description: "创建向导默认选择的协议。", control: "select", options: ["HTTP", "TCP"], value: "HTTP" },
          { key: "autoStartTunnel", label: "创建后自动启动", description: "创建成功后立即启动 Tunnel。", control: "switch", value: false },
        ],
      },
    ],
  },
  {
    id: "appearance",
    label: "Appearance",
    kicker: "Visual style",
    description: "主题、密度和动效偏好。",
    icon: "palette",
    groups: [
      {
        title: "Interface",
        description: "统一现代桌面应用视觉体验。",
        items: [
          { key: "theme", label: "主题", description: "选择深色、浅色或跟随系统。", control: "select", options: ["Dark", "Light", "System"], value: "Dark" },
          { key: "density", label: "界面密度", description: "控制卡片、表格和侧边栏间距。", control: "select", options: ["Comfortable", "Compact"], value: "Comfortable" },
          { key: "reduceMotion", label: "减少动画", description: "降低页面、Dialog 和 Toast 动效。", control: "switch", value: false },
        ],
      },
    ],
  },
  {
    id: "advanced",
    label: "Advanced",
    kicker: "Power user",
    description: "日志、缓存、诊断和实验选项集中到高级区。",
    icon: "sliders",
    groups: [
      {
        title: "Diagnostics",
        description: "默认保持安静，只在需要时打开。",
        items: [
          { key: "verboseLogs", label: "详细日志", description: "输出更多调试信息到 Logs 页面。", control: "switch", value: false },
          { key: "logRetention", label: "日志保留", description: "本地最多保留的日志条数。", control: "number", value: 100000 },
        ],
      },
    ],
  },
  {
    id: "maintenance",
    label: "Maintenance",
    kicker: "Config and cleanup",
    description: "恢复默认设置、导入导出配置、备份配置、重置缓存和清理日志。",
    icon: "sliders",
    groups: [
      {
        title: "Configuration",
        description: "所有配置操作都可视化，不需要手动编辑文件。",
        items: [
          { key: "restoreDefaults", label: "恢复默认设置", description: "将界面偏好恢复为推荐默认值。", control: "action", value: "", icon: "refresh", buttonText: "恢复", action: "restoreDefaults" },
          { key: "exportConfig", label: "导出配置", description: "导出当前设置为 JSON 文件。", control: "action", value: "", icon: "download", buttonText: "导出", action: "exportConfig" },
          { key: "importConfig", label: "导入配置", description: "从 JSON 文件导入设置。", control: "action", value: "", icon: "upload", buttonText: "导入", action: "importConfig" },
          { key: "backupConfig", label: "备份配置", description: "生成带时间戳的配置备份文件。", control: "action", value: "", icon: "save", buttonText: "备份", action: "backupConfig" },
        ],
      },
      {
        title: "Local Data",
        description: "清理不会影响服务端，也不会新增或删除 Tunnel 协议。",
        items: [
          { key: "resetCache", label: "重置缓存", description: "清理欢迎向导、最近服务器和连接历史缓存。", control: "action", value: "", icon: "trash", buttonText: "重置", action: "resetCache" },
          { key: "clearLogs", label: "清理日志", description: "清理本地日志视图缓存；服务端日志不受影响。", control: "action", value: "", icon: "logs", buttonText: "清理", action: "clearLogs" },
        ],
      },
    ],
  },
  {
    id: "about",
    label: "About",
    kicker: "Version and legal",
    description: "版本信息、更新状态和项目链接。",
    icon: "info-circle",
    groups: [
      {
        title: "Gate",
        description: "当前桌面客户端版本。",
        items: [
          { key: "version", label: "Version", description: "当前安装的 Gate Desktop 版本。", control: "readonly", value: "0.1.0" },
          { key: "channel", label: "Channel", description: "当前更新通道。", control: "readonly", value: "Beta Sprint 1" },
        ],
      },
    ],
  },
]

const values = reactive<Record<string, string | number | boolean>>(
  Object.fromEntries(categories.flatMap((category) => category.groups.flatMap((group) => group.items.map((item) => [item.key, item.value])))),
)
const defaultValues = Object.fromEntries(
  categories.flatMap((category) =>
    category.groups.flatMap((group) => group.items.map((item) => [item.key, item.value])),
  ),
) as Record<string, string | number | boolean>

const visibleCategories = computed(() => {
  const keyword = query.value.toLowerCase()
  if (!keyword) return categories
  return categories.filter((category) =>
    [category.label, category.kicker, category.description, ...category.groups.flatMap((group) => [group.title, group.description, ...group.items.flatMap((item) => [item.label, item.description])])]
      .join(" ")
      .toLowerCase()
      .includes(keyword),
  )
})

const selectedCategory = computed(() => visibleCategories.value.find((category) => category.id === activeCategory.value) ?? visibleCategories.value[0])

watch(visibleCategories, (list) => {
  if (!list.some((category) => category.id === activeCategory.value)) {
    activeCategory.value = list[0]?.id ?? "general"
  }
})

function runSettingAction(action?: SettingAction) {
  if (!action) return
  if (action === "restoreDefaults") {
    Object.assign(values, defaultValues)
    toast.success("已恢复默认设置")
  }
  if (action === "exportConfig") {
    downloadConfig("gate-config.json")
    toast.success("配置已导出")
  }
  if (action === "backupConfig") {
    const date = new Date().toISOString().replace(/[:.]/g, "-")
    downloadConfig(`gate-config-backup-${date}.json`)
    toast.success("配置备份已生成")
  }
  if (action === "importConfig") {
    importInput.value?.click()
  }
  if (action === "resetCache") {
    localStorage.removeItem("gate.firstLaunch.completed")
    localStorage.removeItem("gate.recentServers")
    localStorage.removeItem("gate.connectionHistory")
    localStorage.removeItem("gate.welcome.dismissed")
    toast.success("缓存已重置，下次启动会重新显示向导")
  }
  if (action === "clearLogs") {
    localStorage.removeItem("gate.logs.cache")
    toast.success("日志缓存已清理")
  }
}

function downloadConfig(filename: string) {
  const blob = new Blob([JSON.stringify(values, null, 2)], { type: "application/json" })
  const url = URL.createObjectURL(blob)
  const link = document.createElement("a")
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
      const imported = JSON.parse(String(reader.result)) as Record<string, string | number | boolean>
      for (const [key, value] of Object.entries(imported)) {
        if (key in values && ["string", "number", "boolean"].includes(typeof value)) {
          values[key] = value
        }
      }
      toast.success("配置已导入")
    } catch {
      toast.error("导入失败：配置文件不是有效 JSON")
    } finally {
      input.value = ""
    }
  }
  reader.readAsText(file)
}
</script>
