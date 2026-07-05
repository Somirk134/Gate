<!--
  TunnelsPage — 隧道工作区（Docker Desktop 风格三栏）
  ------------------------------------------------------------------
  左：Tunnel List（搜索 / 筛选 / 排序 + 列表项）
  中：Tunnel Workspace（Header + 标签页：Overview / Traffic / Connection /
     Logs / Statistics / Settings / Monitor）
  右：Inspector（实时信息 + 统计 + 日志）

  非传统后台管理布局。支持深链 /tunnels/:tunnelId。
  所有数据来自 Mock，由 useTunnelMonitor 驱动实时刷新。
-->
<template>
  <div class="tunnels-page">
    <!-- 加载态 -->
    <template v-if="isLoading">
      <TunnelLoading :count="8" />
    </template>

    <!-- 错误态 -->
    <GCard v-else-if="isError" variant="plain" padding="lg" style="margin: var(--space-6)">
      <GErrorState
        title="加载失败"
        :message="error || '无法加载隧道列表，请重试。'"
        retry
        @retry="retry"
      />
    </GCard>

    <!-- 空状态 -->
    <TunnelEmpty v-else-if="!hasTunnels" @create="openCreate" />

    <!-- 三栏工作区 -->
    <div v-else class="tunnels-workspace">
      <!-- ============ 左栏：隧道列表 ============ -->
      <aside class="tunnel-list-pane">
        <TunnelToolbar
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

        <div class="tunnel-list__items">
          <template v-if="finalTunnels.length">
            <TunnelCard
              v-for="tunnel in finalTunnels"
              :key="tunnel.id"
              :tunnel="tunnel"
              :active="selectedId === tunnel.id"
              @select="onSelectTunnel"
              @open="onOpenTunnel"
              @contextmenu="onContextmenu"
            />
          </template>
          <div v-else class="tunnel-list__items-empty">
            <GIcon name="search" :size="20" />
            <span>未找到匹配的隧道</span>
          </div>
        </div>

        <div class="tunnel-list__footer">
          <span>{{ finalTunnels.length }} / {{ tunnels.length }} 个隧道</span>
          <span>{{ runningCount }} 运行中</span>
        </div>
      </aside>

      <!-- ============ 中栏：工作区 ============ -->
      <main class="tunnel-workspace-pane">
        <template v-if="selectedTunnel">
          <!-- 详情头部 -->
          <TunnelHeader
            :tunnel="selectedTunnel"
            @start="onStart"
            @stop="onStop"
            @restart="onRestart"
            @clone="onClone"
            @export="onExport"
            @delete="openDelete"
            @toggle-pin="togglePin"
            @toggle-favorite="toggleFavorite"
          />

          <!-- 标签页 -->
          <div class="tunnel-workspace__tabs">
            <button
              v-for="tab in tabs"
              :key="tab.key"
              type="button"
              class="tunnel-workspace__tab"
              :class="{ 'tunnel-workspace__tab--active': activeTab === tab.key }"
              @click="activeTab = tab.key"
            >
              <GIcon :name="tab.icon" :size="13" />
              <span>{{ tab.label }}</span>
            </button>
          </div>

          <!-- 工作区内容 -->
          <div class="tunnel-workspace__content">
            <component
              :is="workspaceComponent"
              :key="selectedTunnel.id"
              :tunnel="selectedTunnel"
              @save="onSettingsSave"
              @export="onLogExport"
              @clear="onLogClear"
              @refresh="onConnectionRefresh"
            />
          </div>
        </template>

        <!-- 未选中占位 -->
        <div v-else class="tunnel-workspace__placeholder">
          <GIcon name="router" :size="40" />
          <span>从左侧选择一个隧道查看详情</span>
          <GButton variant="primary" icon="plus" @click="openCreate">New Tunnel</GButton>
        </div>
      </main>

      <!-- ============ 右栏：Inspector ============ -->
      <aside class="tunnel-inspector-pane">
        <TunnelInspector v-if="selectedTunnel" :tunnel="selectedTunnel" />
        <div v-else class="tunnel-workspace__placeholder">
          <GIcon name="activity" :size="32" />
          <span>实时检查器</span>
        </div>
      </aside>
    </div>

    <!-- 创建/编辑对话框 -->
    <TunnelDialog
      v-model:visible="dialogVisible"
      :tunnel="editingTunnel"
      :projects="projectOptions"
      :server-names="serverNames"
      @submit="handleSubmit"
    />

    <!-- 删除对话框 -->
    <TunnelDeleteDialog
      v-model:visible="deleteDialogVisible"
      :tunnel="deletingTunnel"
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

import TunnelToolbar from "./components/TunnelToolbar.vue"
import TunnelCard from "./components/TunnelCard.vue"
import TunnelHeader from "./components/TunnelHeader.vue"
import TunnelInspector from "./components/TunnelInspector.vue"
import TunnelOverview from "./components/TunnelOverview.vue"
import TunnelTraffic from "./components/TunnelTraffic.vue"
import TunnelConnection from "./components/TunnelConnection.vue"
import TunnelLogs from "./components/TunnelLogs.vue"
import TunnelStatistics from "./components/TunnelStatistics.vue"
import TunnelSettings from "./components/TunnelSettings.vue"
import TunnelMonitor from "./components/TunnelMonitor.vue"
import TunnelEmpty from "./components/TunnelEmpty.vue"
import TunnelLoading from "./components/TunnelLoading.vue"
import TunnelDialog from "./components/TunnelDialog.vue"
import TunnelDeleteDialog from "./components/TunnelDeleteDialog.vue"

import { useTunnel } from "./composables/useTunnel"
import { useTunnelFilter } from "./composables/useTunnelFilter"
import { useTunnelSearch } from "./composables/useTunnelSearch"
import { useTunnelSort } from "./composables/useTunnelSort"
import { useTunnelMonitor } from "./composables/useTunnelMonitor"
import { mockProjects, mockServerNames } from "./mock"
import type {
  Tunnel,
  TunnelFilterType,
  TunnelFormData,
  TunnelSortType,
  TunnelWorkspaceTab,
  SortDirection,
} from "./types"

import "./styles/tunnel.css"

const route = useRoute()
const router = useRouter()
const { toast } = useFeedback()

const {
  tunnels,
  isLoading,
  isError,
  error,
  hasTunnels,
  retry,
  getById,
  create,
  update,
  remove,
  start,
  stop,
  restart,
  clone,
  togglePin,
  toggleFavorite,
  store,
} = useTunnel()

// 启动实时监控
useTunnelMonitor(store)

// ── 工具栏状态 ──
const query = ref("")
const filter = ref<TunnelFilterType>("all")
const sortBy = ref<TunnelSortType>("updatedAt")
const direction = ref<SortDirection>("desc")

// ── 筛选 → 搜索 → 排序 链式处理 ──
const { filtered, counts } = useTunnelFilter(tunnels, filter)
const { results } = useTunnelSearch(filtered, query)
const { sorted } = useTunnelSort(results, sortBy, direction)

const finalTunnels = computed(() => sorted.value)

const runningCount = computed(
  () => tunnels.value.filter((t) => t.status === "running").length,
)

// ── 选中隧道 ──
const selectedId = ref<string | null>(null)
const selectedTunnel = computed(() =>
  selectedId.value ? getById(selectedId.value) : undefined,
)

// 深链支持：/tunnels/:tunnelId
const routeTunnelId = computed(() => (route.params.tunnelId as string) || null)

watch(
  routeTunnelId,
  (id) => {
    if (id && getById(id)) {
      selectedId.value = id
    }
  },
  { immediate: true },
)

// 默认选中第一个
watch(
  tunnels,
  (list) => {
    if (!selectedId.value && list.length) {
      selectedId.value = list[0].id
    }
  },
  { immediate: true },
)

// ── 工作区标签页 ──
const tabs: Array<{ key: TunnelWorkspaceTab; label: string; icon: string }> = [
  { key: "overview", label: "Overview", icon: "info-circle" },
  { key: "traffic", label: "Traffic", icon: "chart-line" },
  { key: "connection", label: "Connection", icon: "link" },
  { key: "logs", label: "Logs", icon: "terminal" },
  { key: "statistics", label: "Statistics", icon: "chart-bar" },
  { key: "monitor", label: "Monitor", icon: "activity" },
  { key: "settings", label: "Settings", icon: "settings" },
]

const activeTab = ref<TunnelWorkspaceTab>("overview")

const workspaceComponent = computed(() => {
  switch (activeTab.value) {
    case "overview":
      return TunnelOverview
    case "traffic":
      return TunnelTraffic
    case "connection":
      return TunnelConnection
    case "logs":
      return TunnelLogs
    case "statistics":
      return TunnelStatistics
    case "monitor":
      return TunnelMonitor
    case "settings":
      return TunnelSettings
    default:
      return TunnelOverview
  }
})

// ── 对话框状态 ──
const dialogVisible = ref(false)
const editingTunnel = ref<Tunnel | null>(null)
const deleteDialogVisible = ref(false)
const deletingTunnel = ref<Tunnel | null>(null)

const projectOptions = mockProjects
const serverNames = mockServerNames

// ── 操作处理 ──
function onSelectTunnel(tunnel: Tunnel) {
  selectedId.value = tunnel.id
}

function onOpenTunnel(tunnel: Tunnel) {
  selectedId.value = tunnel.id
  activeTab.value = "overview"
}

function onContextmenu(tunnel: Tunnel) {
  toast.info(`「${tunnel.name}」右键菜单（预留）`)
}

function onStart() {
  if (!selectedTunnel.value) return
  start(selectedTunnel.value.id)
  toast.success(`正在启动隧道「${selectedTunnel.value.name}」`)
}

function onStop() {
  if (!selectedTunnel.value) return
  stop(selectedTunnel.value.id)
  toast.warning(`已停止隧道「${selectedTunnel.value.name}」`)
}

function onRestart() {
  if (!selectedTunnel.value) return
  restart(selectedTunnel.value.id)
  toast.info(`正在重启隧道「${selectedTunnel.value.name}」`)
}

function onClone() {
  if (!selectedTunnel.value) return
  const cloned = clone(selectedTunnel.value.id)
  if (cloned) {
    toast.success(`已克隆为「${cloned.name}」`)
    selectedId.value = cloned.id
  }
}

function onExport() {
  if (!selectedTunnel.value) return
  toast.info(`导出「${selectedTunnel.value.name}」配置（预留）`)
}

function openCreate() {
  editingTunnel.value = null
  dialogVisible.value = true
}

function openDelete() {
  if (!selectedTunnel.value) return
  deletingTunnel.value = selectedTunnel.value
  deleteDialogVisible.value = true
}

function handleSubmit(form: TunnelFormData, isEdit: boolean) {
  if (isEdit && editingTunnel.value) {
    update(editingTunnel.value.id, form)
    toast.success(`隧道「${form.name}」已更新`)
  } else {
    const created = create(form)
    toast.success(`隧道「${form.name}」已创建`)
    selectedId.value = created.id
  }
  editingTunnel.value = null
}

function handleDelete(tunnel: Tunnel) {
  remove(tunnel.id)
  toast.success(`隧道「${tunnel.name}」已删除`)
  // 选中下一个
  const next = tunnels.value[0]
  selectedId.value = next ? next.id : null
  deletingTunnel.value = null
  if (!next) router.push("/tunnels")
}

function onSettingsSave(id: string, patch: Partial<TunnelFormData>) {
  update(id, patch)
}

function onLogExport() {
  toast.info("导出日志（预留）")
}

function onLogClear() {
  if (!selectedTunnel.value) return
  selectedTunnel.value.logs = []
  toast.success("日志已清空")
}

function onConnectionRefresh() {
  toast.info("连接列表已刷新（Mock）")
}
</script>

<style scoped>
.tunnel-list__items-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-8);
  color: var(--text-tertiary);
  font-size: var(--text-sm);
}
</style>
