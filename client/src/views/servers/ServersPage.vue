<!--
  ServersPage — 服务器工作区（Docker Desktop 风格三栏）
  ------------------------------------------------------------------
  左：Server List（搜索 / 筛选 / 排序 + 列表项）
  中：Server Workspace（Header + 标签页：Overview / Monitor / Health /
     Tunnels / Projects / Traffic / Logs / Statistics / Settings）
  右：Inspector（实时信息 + 统计 + 日志）

  非传统后台管理布局。支持深链 /servers/:serverId。
  所有数据来自 Mock，由 useServerMonitor 驱动实时刷新。

  Server 是资源管理中心：一个客户端可管理多个 Server，
  所有 Tunnel 必须绑定一个 Server。
-->
<template>
  <div class="servers-page">
    <!-- 加载态 -->
    <template v-if="isLoading">
      <ServerLoading :count="8" />
    </template>

    <!-- 错误态 -->
    <GCard v-else-if="isError" variant="plain" padding="lg" style="margin: var(--space-6)">
      <GErrorState
        title="加载失败"
        :message="error || '无法加载服务器列表，请重试。'"
        retry
        @retry="retry"
      />
    </GCard>

    <!-- 空状态 -->
    <ServerEmpty v-else-if="!hasServers" @create="openCreate" />

    <!-- 三栏工作区 -->
    <div v-else class="servers-workspace">
      <!-- ============ 左栏：服务器列表 ============ -->
      <aside class="server-list-pane">
        <ServerToolbar
          :query="query"
          :filter="filter"
          :sort-by="sortBy"
          :direction="direction"
          :counts="counts"
          @update:query="query = $event"
          @update:filter="filter = $event"
          @update:sort-by="sortBy = $event"
          @update:direction="direction = $event"
          @create="openCreate"
        />

        <div class="server-list__items">
          <template v-if="finalServers.length">
            <ServerCard
              v-for="server in finalServers"
              :key="server.id"
              :server="server"
              :active="selectedId === server.id"
              @select="onSelectServer"
              @open="onOpenServer"
              @contextmenu="onContextmenu"
            />
          </template>
          <div v-else class="server-list__items-empty">
            <GIcon name="search" :size="20" />
            <span>未找到匹配的服务器</span>
          </div>
        </div>

        <div class="server-list__footer">
          <span>{{ finalServers.length }} / {{ servers.length }} 台服务器</span>
          <span>{{ onlineCount }} 在线</span>
        </div>
      </aside>

      <!-- ============ 中栏：工作区 ============ -->
      <main class="server-workspace-pane">
        <template v-if="selectedServer">
          <!-- 详情头部 -->
          <ServerHeader
            :server="selectedServer"
            @connect="onConnect"
            @disconnect="onDisconnect"
            @restart="onRestart"
            @check-health="onCheckHealth"
            @edit="openEdit"
            @delete="openDelete"
            @toggle-favorite="toggleFavorite"
          />

          <!-- 标签页 -->
          <div class="server-workspace__tabs">
            <button
              v-for="tab in tabs"
              :key="tab.key"
              type="button"
              class="server-workspace__tab"
              :class="{ 'server-workspace__tab--active': activeTab === tab.key }"
              @click="activeTab = tab.key"
            >
              <GIcon :name="tab.icon" :size="13" />
              <span>{{ tab.label }}</span>
            </button>
          </div>

          <!-- 工作区内容 -->
          <div class="server-workspace__content">
            <component
              :is="workspaceComponent"
              :key="selectedServer.id"
              :server="selectedServer"
              :tunnels="serverTunnels"
              :projects="serverProjects"
              @save="onSettingsSave"
              @export="onLogExport"
              @clear="onLogClear"
              @recheck="onRecheckHealth"
              @create-tunnel="onCreateTunnel"
              @start-tunnel="onStartTunnel"
              @stop-tunnel="onStopTunnel"
              @view-tunnel="onViewTunnel"
              @view-project="onViewProject"
            />
          </div>
        </template>

        <!-- 未选中占位 -->
        <div v-else class="server-workspace__placeholder">
          <GIcon name="servers" :size="40" />
          <span>从左侧选择一台服务器查看详情</span>
          <GButton variant="primary" icon="plus" @click="openCreate">Add Server</GButton>
        </div>
      </main>

      <!-- ============ 右栏：Inspector ============ -->
      <aside class="server-inspector-pane">
        <ServerInspector v-if="selectedServer" :server="selectedServer" />
        <div v-else class="server-workspace__placeholder">
          <GIcon name="activity" :size="32" />
          <span>实时检查器</span>
        </div>
      </aside>
    </div>

    <!-- 创建/编辑对话框 -->
    <ServerDialog
      v-model:visible="dialogVisible"
      :server="editingServer"
      @submit="handleSubmit"
    />

    <!-- 删除对话框 -->
    <ServerDeleteDialog
      v-model:visible="deleteDialogVisible"
      :server="deletingServer"
      @confirm="handleDelete"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from "vue"
import { useRoute, useRouter } from "vue-router"
import { useFeedback } from "@composables/useFeedback"
import GIcon from "@components/icons/GIcon.vue"
import GButton from "@components/base/GButton.vue"
import GCard from "@components/base/GCard.vue"
import GErrorState from "@components/feedback/GErrorState.vue"

import ServerToolbar from "./components/ServerToolbar.vue"
import ServerCard from "./components/ServerCard.vue"
import ServerHeader from "./components/ServerHeader.vue"
import ServerInspector from "./components/ServerInspector.vue"
import ServerOverview from "./components/ServerOverview.vue"
import ServerMonitor from "./components/ServerMonitor.vue"
import ServerHealth from "./components/ServerHealth.vue"
import ServerTunnels from "./components/ServerTunnels.vue"
import ServerProjects from "./components/ServerProjects.vue"
import ServerTraffic from "./components/ServerTraffic.vue"
import ServerLogs from "./components/ServerLogs.vue"
import ServerStatistics from "./components/ServerStatistics.vue"
import ServerSettings from "./components/ServerSettings.vue"
import ServerEmpty from "./components/ServerEmpty.vue"
import ServerLoading from "./components/ServerLoading.vue"
import ServerDialog from "./components/ServerDialog.vue"
import ServerDeleteDialog from "./components/ServerDeleteDialog.vue"
import type { ServerTunnelItem } from "./components/ServerTunnels.vue"
import type { ServerProjectItem } from "./components/ServerProjects.vue"

import { useServer } from "./composables/useServer"
import { useServerFilter } from "./composables/useServerFilter"
import { useServerSearch } from "./composables/useServerSearch"
import { useServerSort } from "./composables/useServerSort"
import { useServerMonitor } from "./composables/useServerMonitor"
import { isOnlineStatus } from "./utils"
import type {
  Server,
  ServerFilterType,
  ServerFormData,
  ServerSortType,
  ServerWorkspaceTab,
  SortDirection,
} from "./types"

import "./styles/server.css"

const route = useRoute()
const router = useRouter()
const { toast } = useFeedback()

const {
  servers,
  isLoading,
  isError,
  error,
  hasServers,
  retry,
  getById,
  create,
  update,
  remove,
  connect,
  disconnect,
  restart,
  checkHealth,
  toggleFavorite,
  store,
} = useServer()

// 启动实时监控
useServerMonitor(store)

// ── 工具栏状态 ──
const query = ref("")
const filter = ref<ServerFilterType>("all")
const sortBy = ref<ServerSortType>("name")
const direction = ref<SortDirection>("asc")

// ── 筛选 → 搜索 → 排序 链式处理 ──
const { filtered, counts } = useServerFilter(servers, filter)
const { results } = useServerSearch(filtered, query)
const { sorted } = useServerSort(results, sortBy, direction)

const finalServers = computed(() => sorted.value)

const onlineCount = computed(
  () => servers.value.filter((s) => isOnlineStatus(s.status)).length,
)

// ── 选中服务器 ──
const selectedId = ref<string | null>(null)
const selectedServer = computed(() =>
  selectedId.value ? getById(selectedId.value) : undefined,
)

// 深链支持：/servers/:serverId
const routeServerId = computed(() => (route.params.serverId as string) || null)

watch(
  routeServerId,
  (id) => {
    if (id && getById(id)) {
      selectedId.value = id
    }
  },
  { immediate: true },
)

// 默认选中第一个
watch(
  servers,
  (list) => {
    if (!selectedId.value && list.length) {
      selectedId.value = list[0].id
    }
  },
  { immediate: true },
)

// ── 工作区标签页 ──
const tabs: Array<{ key: ServerWorkspaceTab; label: string; icon: string }> = [
  { key: "overview", label: "Overview", icon: "info-circle" },
  { key: "monitor", label: "Monitor", icon: "activity" },
  { key: "health", label: "Health", icon: "shield-check" },
  { key: "tunnels", label: "Tunnels", icon: "router" },
  { key: "projects", label: "Projects", icon: "package" },
  { key: "traffic", label: "Traffic", icon: "chart-line" },
  { key: "logs", label: "Logs", icon: "terminal" },
  { key: "statistics", label: "Statistics", icon: "chart-bar" },
  { key: "settings", label: "Settings", icon: "settings" },
]

const activeTab = ref<ServerWorkspaceTab>("overview")

const workspaceComponent = computed(() => {
  switch (activeTab.value) {
    case "overview":
      return ServerOverview
    case "monitor":
      return ServerMonitor
    case "health":
      return ServerHealth
    case "tunnels":
      return ServerTunnels
    case "projects":
      return ServerProjects
    case "traffic":
      return ServerTraffic
    case "logs":
      return ServerLogs
    case "statistics":
      return ServerStatistics
    case "settings":
      return ServerSettings
    default:
      return ServerOverview
  }
})

// ── Mock：生成属于当前服务器的 Tunnel 列表 ──
const tunnelSeedNames = [
  "api-gateway", "web-frontend", "ssh-tunnel", "postgres-db",
  "redis-cache", "grafana-ui", "admin-panel", "websocket-chat",
]
const tunnelIcons = ["globe", "globe", "router", "database", "database", "chart-line", "globe", "link"]
const tunnelColors = ["#5B8DEF", "#5B8DEF", "#22C55E", "#EF4444", "#EF4444", "#06B6D4", "#5B8DEF", "#F59E0B"]
const tunnelProtocols = ["http", "http", "tcp", "tcp", "tcp", "http", "http", "tcp"]

const serverTunnels = computed<ServerTunnelItem[]>(() => {
  const s = selectedServer.value
  if (!s) return []
  const count = s.statistics.tunnelCount
  const list: ServerTunnelItem[] = []
  for (let i = 0; i < count; i++) {
    const idx = i % tunnelSeedNames.length
    const running = isOnlineStatus(s.status) && i % 3 !== 0
    list.push({
      id: `${s.id}-t${i + 1}`,
      name: tunnelSeedNames[idx],
      protocol: tunnelProtocols[idx],
      publicAddr: `${s.publicIp}:${8080 + i}`,
      running,
      icon: tunnelIcons[idx],
      color: tunnelColors[idx],
    })
  }
  return list
})

// ── Mock：生成属于当前服务器的 Project 列表 ──
const projectSeedNames = [
  "My API Service", "Web App Frontend", "Dev Environment",
  "Monitoring Stack", "IoT Gateway", "Database Cluster",
]
const projectColors = ["#5B8DEF", "#06B6D4", "#7C6FF2", "#22C55E", "#F59E0B", "#EF4444"]

const serverProjects = computed<ServerProjectItem[]>(() => {
  const s = selectedServer.value
  if (!s) return []
  const count = s.statistics.projectCount
  const list: ServerProjectItem[] = []
  for (let i = 0; i < count; i++) {
    const idx = i % projectSeedNames.length
    list.push({
      id: `${s.id}-p${i + 1}`,
      name: projectSeedNames[idx],
      tunnelCount: Math.max(1, Math.floor(s.statistics.tunnelCount / Math.max(count, 1))),
      remark: ["核心 API 服务", "前端应用", "开发环境", "监控体系", "物联网网关", "数据库集群"][idx] ?? "",
      color: projectColors[idx],
    })
  }
  return list
})

// ── 对话框状态 ──
const dialogVisible = ref(false)
const editingServer = ref<Server | null>(null)
const deleteDialogVisible = ref(false)
const deletingServer = ref<Server | null>(null)

// ── 操作处理 ──
function onSelectServer(server: Server) {
  selectedId.value = server.id
}

function onOpenServer(server: Server) {
  selectedId.value = server.id
  activeTab.value = "overview"
}

function onContextmenu(server: Server) {
  toast.info(`「${server.name}」右键菜单（预留）`)
}

function onConnect() {
  if (!selectedServer.value) return
  connect(selectedServer.value.id)
  toast.success(`正在连接服务器「${selectedServer.value.name}」`)
}

function onDisconnect() {
  if (!selectedServer.value) return
  disconnect(selectedServer.value.id)
  toast.warning(`已断开服务器「${selectedServer.value.name}」`)
}

function onRestart() {
  if (!selectedServer.value) return
  restart(selectedServer.value.id)
  toast.info(`正在重启服务器「${selectedServer.value.name}」`)
}

function onCheckHealth() {
  if (!selectedServer.value) return
  checkHealth(selectedServer.value.id)
  toast.info(`正在检查「${selectedServer.value.name}」健康状态…`)
}

function onRecheckHealth() {
  onCheckHealth()
}

function openCreate() {
  editingServer.value = null
  dialogVisible.value = true
}

function openEdit() {
  if (!selectedServer.value) return
  editingServer.value = selectedServer.value
  dialogVisible.value = true
}

function openDelete() {
  if (!selectedServer.value) return
  deletingServer.value = selectedServer.value
  deleteDialogVisible.value = true
}

function handleSubmit(form: ServerFormData, isEdit: boolean) {
  if (isEdit && editingServer.value) {
    update(editingServer.value.id, form)
    toast.success(`服务器「${form.name}」已更新`)
  } else {
    const created = create(form)
    toast.success(`服务器「${form.name}」已添加`)
    selectedId.value = created.id
  }
  editingServer.value = null
}

function handleDelete(server: Server) {
  remove(server.id)
  toast.success(`服务器「${server.name}」已删除`)
  // 选中下一个
  const next = servers.value[0]
  selectedId.value = next ? next.id : null
  deletingServer.value = null
  if (!next) router.push("/servers")
}

function onSettingsSave(id: string, patch: Partial<ServerFormData>) {
  update(id, patch)
}

function onLogExport() {
  toast.info("导出日志（预留）")
}

function onLogClear() {
  if (!selectedServer.value) return
  selectedServer.value.logs = []
  toast.success("日志已清空")
}

function onCreateTunnel() {
  toast.info("跳转到 Tunnel 模块创建（预留）")
}

function onStartTunnel(tunnel: ServerTunnelItem) {
  tunnel.running = true
  toast.success(`已启动 Tunnel「${tunnel.name}」`)
}

function onStopTunnel(tunnel: ServerTunnelItem) {
  tunnel.running = false
  toast.warning(`已停止 Tunnel「${tunnel.name}」`)
}

function onViewTunnel(tunnel: ServerTunnelItem) {
  toast.info(`查看 Tunnel「${tunnel.name}」详情（预留）`)
}

function onViewProject(project: ServerProjectItem) {
  toast.info(`进入 Project「${project.name}」（预留）`)
}
</script>

<style scoped>
.server-list__items-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-8);
  color: var(--text-tertiary);
  font-size: var(--text-sm);
}
</style>
