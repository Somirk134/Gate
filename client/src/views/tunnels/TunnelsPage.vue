<template>
  <section class="tunnels-page">
    <header class="tunnels-hero">
      <div>
        <p>{{ t('tunnel.workspace') }}</p>
        <h1>{{ t('tunnel.title') }}</h1>
        <span>{{
          t('tunnel.summary', {
            running: runningCount,
            total: tunnels.length,
            speed: formatSpeed(totalSpeed),
          })
        }}</span>
      </div>
      <div class="tunnels-hero__actions">
        <GButton variant="secondary" icon="globe" @click="router.push('/tunnels/http')">
          HTTP
        </GButton>
        <GButton variant="primary" icon="plus" @click="openCreate">
          {{ t('tunnel.create') }}
        </GButton>
      </div>
    </header>

    <TunnelLoading v-if="isLoading" :count="8" />

    <GCard v-else-if="isError" variant="plain" padding="lg">
      <GErrorState
        :title="t('tunnel.loadFailed')"
        :message="error || t('tunnel.loadFailedMessage')"
        retry
        @retry="retry" />
    </GCard>

    <div v-else-if="!hasTunnels" class="tunnel-empty-state">
      <div class="empty-illustration">
        <GIcon name="router" :size="34" />
      </div>
      <h2>{{ t('tunnel.emptyTitle') }}</h2>
      <p>{{ t('tunnel.emptyDesc') }}</p>
      <GButton variant="primary" icon="plus" @click="openCreate">
        {{ t('tunnel.createFirst') }}
      </GButton>
    </div>

    <template v-else>
      <div class="tunnel-toolbar">
        <label class="toolbar-search">
          <GIcon name="search" :size="15" />
          <input v-model.trim="query" :placeholder="t('tunnel.searchPlaceholder')" />
        </label>
        <select v-model="filter">
          <option value="all">{{ t('tunnel.filters.all') }}</option>
          <option value="running">{{ t('tunnel.filters.running') }}</option>
          <option value="stopped">{{ t('tunnel.filters.stopped') }}</option>
          <option value="http">HTTP</option>
          <option value="tcp">TCP</option>
          <option value="favorite">{{ t('tunnel.filters.favorite') }}</option>
          <option value="recent">{{ t('tunnel.filters.recent') }}</option>
        </select>
        <select v-model="sortBy">
          <option value="updatedAt">{{ t('tunnel.sort.updatedAt') }}</option>
          <option value="name">{{ t('tunnel.sort.name') }}</option>
          <option value="status">{{ t('tunnel.sort.status') }}</option>
          <option value="traffic">{{ t('tunnel.sort.traffic') }}</option>
          <option value="connections">{{ t('tunnel.sort.connections') }}</option>
        </select>
        <button
          type="button"
          class="sort-direction"
          @click="direction = direction === 'asc' ? 'desc' : 'asc'">
          <GIcon name="arrow-up-down" :size="15" />
          {{ direction === 'asc' ? t('tunnel.sort.asc') : t('tunnel.sort.desc') }}
        </button>
      </div>

      <div class="tunnel-workspace">
        <aside class="tunnel-list" :aria-label="t('tunnel.listAria')">
          <div class="tunnel-list__header">
            <strong>{{ t('tunnel.resultCount', { count: finalTunnels.length }) }}</strong>
            <span>{{ query ? t('tunnel.matching', { query }) : t('tunnel.ready') }}</span>
          </div>

          <button
            v-for="tunnel in finalTunnels"
            :key="tunnel.id"
            type="button"
            class="tunnel-row"
            :class="{ active: selectedId === tunnel.id }"
            @click="selectTunnel(tunnel.id)">
            <span class="tunnel-row__status" :class="`is-${statusTone(tunnel.status)}`" />
            <div class="tunnel-row__main">
              <strong>{{ tunnel.name }}</strong>
              <small
                >{{ tunnel.protocol.toUpperCase() }} · {{ tunnel.localHost }}:{{
                  tunnel.localPort
                }}</small
              >
            </div>
            <div class="tunnel-row__meta">
              <span>{{
                formatSpeed(tunnel.traffic.downloadSpeed + tunnel.traffic.uploadSpeed)
              }}</span>
              <small>{{
                t('tunnel.connectionUnit', { count: tunnel.statistics.connections })
              }}</small>
            </div>
          </button>

          <div v-if="!finalTunnels.length" class="tunnel-list__empty">
            <GIcon name="search" :size="24" />
            <span>{{ t('tunnel.noMatching') }}</span>
          </div>
        </aside>

        <main class="tunnel-detail" aria-live="polite">
          <template v-if="selectedTunnel">
            <div class="detail-header">
              <div>
                <div class="detail-title-row">
                  <span :class="`is-${statusTone(selectedTunnel.status)}`" />
                  <h2>{{ selectedTunnel.name }}</h2>
                </div>
                <p>{{ selectedTunnel.projectName }} · {{ selectedTunnel.serverName }}</p>
              </div>
              <div class="detail-actions">
                <GButton
                  v-if="canStart(selectedTunnel.status)"
                  variant="primary"
                  icon="play"
                  @click="startSelected">
                  {{ t('tunnel.start') }}
                </GButton>
                <GButton v-else variant="secondary" icon="pause" @click="stopSelected">
                  {{ t('tunnel.stop') }}
                </GButton>
                <button
                  type="button"
                  class="icon-action"
                  :class="{ active: selectedTunnel.favorite }"
                  @click="toggleFavorite(selectedTunnel.id)">
                  <GIcon name="star" :size="16" />
                </button>
                <button type="button" class="icon-action" @click="deleteSelected">
                  <GIcon name="trash" :size="16" />
                </button>
              </div>
            </div>

            <div class="detail-metrics">
              <article>
                <span>{{ t('tunnel.detail.publicAddress') }}</span>
                <strong>{{ selectedTunnel.publicAddr }}</strong>
              </article>
              <article>
                <span>{{ t('tunnel.detail.todayTraffic') }}</span>
                <strong>{{
                  formatBytes(
                    selectedTunnel.traffic.todayUpload + selectedTunnel.traffic.todayDownload,
                  )
                }}</strong>
              </article>
              <article>
                <span>{{ t('tunnel.detail.realtimeSpeed') }}</span>
                <strong>{{
                  formatSpeed(
                    selectedTunnel.traffic.uploadSpeed + selectedTunnel.traffic.downloadSpeed,
                  )
                }}</strong>
              </article>
              <article>
                <span>{{ t('tunnel.detail.uptime') }}</span>
                <strong>{{ formatDuration(selectedTunnel.statistics.uptime) }}</strong>
              </article>
            </div>

            <div class="test-url-panel">
              <div>
                <span>{{ t('tunnel.detail.localTestUrl') }}</span>
                <strong>{{ testUrl(selectedTunnel) }}</strong>
              </div>
              <div class="test-url-panel__actions">
                <GButton variant="secondary" icon="copy" @click="copyTestUrl(selectedTunnel)">
                  {{ t('tunnel.detail.copy') }}
                </GButton>
                <GButton
                  v-if="canOpenTestUrl(selectedTunnel)"
                  variant="primary"
                  icon="external-link"
                  @click="openTestUrl(selectedTunnel)">
                  {{ t('tunnel.detail.open') }}
                </GButton>
              </div>
            </div>

            <div class="detail-grid">
              <section class="detail-card">
                <div class="detail-card__heading">
                  <h3>{{ t('tunnel.detail.path') }}</h3>
                  <GIcon name="link" :size="16" />
                </div>
                <dl class="path-list">
                  <div>
                    <dt>{{ t('tunnel.detail.local') }}</dt>
                    <dd>{{ selectedTunnel.localHost }}:{{ selectedTunnel.localPort }}</dd>
                  </div>
                  <div>
                    <dt>{{ t('tunnel.detail.public') }}</dt>
                    <dd>{{ selectedTunnel.publicAddr }}</dd>
                  </div>
                  <div>
                    <dt>{{ t('tunnel.detail.protocol') }}</dt>
                    <dd>{{ selectedTunnel.protocol.toUpperCase() }}</dd>
                  </div>
                  <div>
                    <dt>{{ t('tunnel.detail.status') }}</dt>
                    <dd>{{ statusLabel(selectedTunnel.status) }}</dd>
                  </div>
                </dl>
              </section>

              <section class="detail-card">
                <div class="detail-card__heading">
                  <h3>{{ t('tunnel.detail.tags') }}</h3>
                  <GIcon name="tag" :size="16" />
                </div>
                <div class="tag-list">
                  <span v-for="tag in selectedTunnel.tags" :key="tag">{{ tagLabel(tag) }}</span>
                  <span v-if="!selectedTunnel.tags.length">{{ t('tunnel.detail.noTags') }}</span>
                </div>
              </section>
            </div>

            <section class="detail-card detail-card--logs">
              <div class="detail-card__heading">
                <h3>{{ t('tunnel.detail.recentLogs') }}</h3>
                <button type="button" @click="activeLogTunnel = selectedTunnel.id">
                  <GIcon name="refresh" :size="14" />
                </button>
              </div>
              <div class="mini-log-list">
                <article v-for="log in selectedTunnel.logs.slice(-6).reverse()" :key="log.id">
                  <span :class="`is-${log.level}`">{{ log.level }}</span>
                  <p>{{ log.message }}</p>
                  <small>{{ formatLogTime(log.timestamp) }}</small>
                </article>
                <div v-if="!selectedTunnel.logs.length" class="mini-empty">
                  <GIcon name="logs" :size="22" />
                  <span>{{ t('tunnel.detail.noData') }}</span>
                </div>
              </div>
            </section>
          </template>

          <div v-else class="tunnel-detail__placeholder">
            <GIcon name="router" :size="34" />
            <span>{{ t('tunnel.selectPrompt') }}</span>
          </div>
        </main>
      </div>
    </template>

    <TunnelCreateWizard
      v-model:visible="wizardVisible"
      :projects="projectOptions"
      :server-names="serverNames"
      :default-project-id="requestedProjectId"
      @submit="handleCreate" />
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'
import { useFeedback } from '@composables/useFeedback'
import { isTauri } from '@tauri-apps/api/core'
import { open as openExternalUrl } from '@tauri-apps/plugin-shell'
import GButton from '@components/base/GButton.vue'
import GCard from '@components/base/GCard.vue'
import GIcon from '@components/icons/GIcon.vue'
import GErrorState from '@components/feedback/GErrorState.vue'
import TunnelLoading from './components/TunnelLoading.vue'
import TunnelCreateWizard from './components/TunnelCreateWizard.vue'
import { useTunnel } from './composables/useTunnel'
import { useTunnelMonitor } from './composables/useTunnelMonitor'
import { useServerStore } from '@views/servers'
import { useProjectStore } from '@views/projects/store/project'
import type {
  SortDirection,
  Tunnel,
  TunnelFilterType,
  TunnelFormData,
  TunnelSortType,
  TunnelStatus,
} from './types'
import './styles/tunnel.css'

const route = useRoute()
const router = useRouter()
const { t, te, locale } = useI18n()
const { toast, notify, confirm, confirmDanger } = useFeedback()
const {
  tunnels,
  isLoading,
  isError,
  error,
  hasTunnels,
  retry,
  getById,
  create,
  remove,
  start,
  stop,
  toggleFavorite,
  store,
} = useTunnel()
const serverStore = useServerStore()
const projectStore = useProjectStore()

useTunnelMonitor(store)

const query = ref('')
const filter = ref<TunnelFilterType>('all')
const sortBy = ref<TunnelSortType>('updatedAt')
const direction = ref<SortDirection>('desc')
const selectedId = ref<string | null>(null)
const wizardVisible = ref(false)
const activeLogTunnel = ref('')
const requestedProjectId = ref('')
const projectOptions = computed(() =>
  projectStore.projects.map((project) => ({ id: project.id, name: project.name })),
)
const serverNames = computed(() => serverStore.onlineServers.map((server) => server.name))

const runningCount = computed(
  () => tunnels.value.filter((tunnel) => canStart(tunnel.status) === false).length,
)
const totalSpeed = computed(() =>
  tunnels.value.reduce(
    (sum, tunnel) => sum + tunnel.traffic.downloadSpeed + tunnel.traffic.uploadSpeed,
    0,
  ),
)

const finalTunnels = computed(() => {
  const keyword = query.value.toLowerCase()
  const filtered = tunnels.value.filter((tunnel) => {
    const matchesFilter =
      filter.value === 'all' ||
      tunnel.protocol === filter.value ||
      (filter.value === 'running' && !canStart(tunnel.status)) ||
      (filter.value === 'stopped' && canStart(tunnel.status)) ||
      (filter.value === 'favorite' && tunnel.favorite) ||
      filter.value === 'recent'
    const matchesQuery =
      !keyword ||
      [
        tunnel.name,
        tunnel.protocol,
        tunnel.projectName,
        tunnel.serverName,
        tunnel.publicAddr,
        ...tunnel.tags,
      ]
        .join(' ')
        .toLowerCase()
        .includes(keyword)
    return matchesFilter && matchesQuery
  })

  const sorted = [...filtered].sort((a, b) => {
    const modifier = direction.value === 'asc' ? 1 : -1
    if (sortBy.value === 'name') return a.name.localeCompare(b.name) * modifier
    if (sortBy.value === 'status') return (statusOrder(a.status) - statusOrder(b.status)) * modifier
    if (sortBy.value === 'traffic') {
      return (trafficTotal(a) - trafficTotal(b)) * modifier
    }
    if (sortBy.value === 'connections') {
      return (a.statistics.connections - b.statistics.connections) * modifier
    }
    return (new Date(a.updatedAt).getTime() - new Date(b.updatedAt).getTime()) * modifier
  })

  return filter.value === 'recent' ? sorted.slice(0, 10) : sorted
})

const selectedTunnel = computed(() => (selectedId.value ? getById(selectedId.value) : undefined))

watch(
  finalTunnels,
  (list) => {
    if (!selectedId.value || !list.some((tunnel) => tunnel.id === selectedId.value)) {
      selectedId.value = list[0]?.id ?? null
    }
  },
  { immediate: true },
)

watch(
  () => serverStore.status,
  (status) => {
    if (status === 'idle') {
      void serverStore.load()
    }
  },
  { immediate: true },
)

onMounted(() => {
  if (projectStore.status === 'idle') {
    void projectStore.load()
  }
})

watch(
  () => route.query.create,
  (value) => {
    if (value === '1') {
      requestedProjectId.value =
        typeof route.query.projectId === 'string' ? route.query.projectId : ''
      void openCreate()
      void router.replace({ path: '/tunnels' })
    }
  },
  { immediate: true },
)

function selectTunnel(id: string) {
  selectedId.value = id
}

async function openCreate() {
  if (!projectStore.projects.length) {
    const shouldContinue = await promptForProject()
    if (!shouldContinue) return
  }

  if (!serverNames.value.length) {
    toast.warning(t('tunnel.notifications.needServer'))
    void router.push('/servers')
    return
  }
  wizardVisible.value = true
}

async function handleCreate(form: TunnelFormData) {
  try {
    const created = await create(form)
    if (form.projectId) {
      await projectStore.addTunnel(form.projectId, created.id)
    }
    requestedProjectId.value = ''
    selectedId.value = created.id
    toast.success(t('tunnel.notifications.saved', { name: created.name }))
  } catch (err) {
    notify.error(t('tunnel.notifications.createFailed'), errorMessage(err), 10000)
  }
}

function promptForProject() {
  return new Promise<boolean>((resolve) => {
    confirm({
      title: t('tunnel.notifications.createProject'),
      content: t('tunnel.notifications.noProjectPrompt'),
      confirmText: t('tunnel.notifications.createDefaultProject'),
      cancelText: t('tunnel.notifications.later'),
      onConfirm: async () => {
        try {
          await projectStore.createDefaultProject()
          toast.success(t('tunnel.notifications.defaultProjectCreated'))
          resolve(true)
        } catch (err) {
          notify.error(t('tunnel.notifications.defaultProjectFailed'), errorMessage(err), 10000)
          resolve(false)
        }
      },
      onCancel: () => resolve(true),
    })
  })
}

async function startSelected() {
  if (!selectedTunnel.value) return
  try {
    await start(selectedTunnel.value.id)
    toast.success(t('tunnel.notifications.started', { name: selectedTunnel.value.name }))
  } catch (err) {
    notify.error(t('tunnel.notifications.startFailed'), errorMessage(err), 12000)
  }
}

function stopSelected() {
  const tunnel = selectedTunnel.value
  if (!tunnel) return
  confirm({
    title: t('tunnel.notifications.stopTitle'),
    content: t('tunnel.notifications.stopContent', { name: tunnel.name }),
    confirmText: t('tunnel.stop'),
    onConfirm: async () => {
      try {
        await stop(tunnel.id)
        toast.warning(t('tunnel.notifications.stopped', { name: tunnel.name }))
      } catch (err) {
        notify.error(t('tunnel.notifications.stopFailed'), errorMessage(err), 10000)
      }
    },
  })
}

function deleteSelected() {
  const tunnel = selectedTunnel.value
  if (!tunnel) return
  confirmDanger({
    title: t('tunnel.notifications.deleteTitle'),
    content: t('tunnel.notifications.deleteContent', { name: tunnel.name }),
    confirmText: t('common.delete'),
    onConfirm: async () => {
      try {
        await remove(tunnel.id)
        selectedId.value = finalTunnels.value[0]?.id ?? null
        toast.success(t('tunnel.notifications.deleted', { name: tunnel.name }))
      } catch (err) {
        notify.error(t('tunnel.notifications.deleteFailed'), errorMessage(err), 10000)
      }
    },
  })
}

function canStart(status: TunnelStatus) {
  return (
    status === 'stopped' || status === 'offline' || status === 'error' || status === 'disconnected'
  )
}

function statusTone(status: TunnelStatus) {
  if (status === 'running') return 'online'
  if (status === 'error' || status === 'disconnected') return 'error'
  if (status === 'stopped' || status === 'offline') return 'offline'
  return 'warning'
}

function statusLabel(status: TunnelStatus) {
  return t(`tunnel.statusLabels.${status}`)
}

function statusOrder(status: TunnelStatus) {
  const order: Record<TunnelStatus, number> = {
    running: 0,
    connecting: 1,
    starting: 2,
    restarting: 3,
    stopping: 4,
    error: 5,
    disconnected: 6,
    stopped: 7,
    offline: 8,
  }
  return order[status]
}

function trafficTotal(tunnel: Tunnel) {
  return tunnel.traffic.totalUpload + tunnel.traffic.totalDownload
}

function testUrl(tunnel: Tunnel): string {
  if (tunnel.protocol === 'http') return `http://127.0.0.1:${tunnel.remotePort}/`
  if (tunnel.protocol === 'https') return `https://127.0.0.1:${tunnel.remotePort}/`
  return `127.0.0.1:${tunnel.remotePort}`
}

function canOpenTestUrl(tunnel: Tunnel): boolean {
  return tunnel.protocol === 'http' || tunnel.protocol === 'https'
}

async function copyTestUrl(tunnel: Tunnel) {
  await navigator.clipboard.writeText(testUrl(tunnel))
  toast.success(t('tunnel.notifications.testUrlCopied'))
}

async function openTestUrl(tunnel: Tunnel) {
  const url = testUrl(tunnel)
  try {
    if (isTauri()) {
      await openExternalUrl(url)
      return
    }

    const target = window.open(url, '_blank', 'noopener,noreferrer')
    if (!target) throw new Error(t('tunnel.notifications.popupBlocked'))
  } catch (err) {
    toast.error(err instanceof Error ? err.message : t('tunnel.notifications.openUrlFailed'))
  }
}

function errorMessage(err: unknown): string {
  if (typeof err === 'string') return err
  if (err instanceof Error && err.message) return err.message
  if (err && typeof err === 'object' && 'message' in err) {
    const message = (err as { message?: unknown }).message
    if (typeof message === 'string' && message.trim()) return message
  }
  return t('tunnel.notifications.configCheck')
}

function formatBytes(bytes: number): string {
  if (!Number.isFinite(bytes) || bytes <= 0) return '0 B'
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  const index = Math.min(units.length - 1, Math.floor(Math.log(bytes) / Math.log(1024)))
  const value = bytes / 1024 ** index
  return `${value.toFixed(value >= 10 || index === 0 ? 0 : 1)} ${units[index]}`
}

function formatSpeed(bytesPerSecond: number): string {
  return `${formatBytes(bytesPerSecond)}/s`
}

function formatDuration(seconds: number): string {
  if (seconds <= 0) return t('common.emptyValue')
  const day = Math.floor(seconds / 86400)
  const hour = Math.floor((seconds % 86400) / 3600)
  const minute = Math.floor((seconds % 3600) / 60)
  if (day) return t('common.time.shortDaysHours', { days: day, hours: hour })
  if (hour) return t('common.time.shortHoursMinutes', { hours: hour, minutes: minute })
  return t('common.time.shortMinutes', { count: Math.max(1, minute) })
}

function tagLabel(tag: string): string {
  const key = `tunnel.tags.${tag}`
  return te(key) ? t(key) : tag
}

function formatLogTime(timestamp: number): string {
  return new Intl.DateTimeFormat(locale.value === 'en-US' ? 'en-US' : 'zh-CN', {
    hour: '2-digit',
    minute: '2-digit',
    second: '2-digit',
  }).format(timestamp)
}
</script>

<style scoped>
.tunnels-page {
  width: min(100%, var(--content-max-width));
  height: 100%;
  min-height: 0;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: var(--space-4);
}

.tunnels-hero {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-4);
  flex-shrink: 0;
}

.tunnels-hero__actions {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  flex-shrink: 0;
}

.tunnels-hero p {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
  text-transform: uppercase;
}

.tunnels-hero h1 {
  margin-top: 2px;
  font-size: var(--text-3xl);
  font-weight: var(--weight-semibold);
  letter-spacing: 0;
}

.tunnels-hero span {
  display: block;
  margin-top: var(--space-1);
  color: var(--text-secondary);
}

.tunnel-toolbar {
  display: grid;
  grid-template-columns: minmax(240px, 1fr) 136px 148px auto;
  gap: var(--space-2);
  flex-shrink: 0;
}

.toolbar-search,
.tunnel-toolbar select,
.sort-direction {
  height: 36px;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  background: var(--bg-input);
  color: var(--text-primary);
}

.toolbar-search {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: 0 var(--space-3);
  color: var(--text-tertiary);
}

.toolbar-search:focus-within {
  border-color: var(--color-primary);
  box-shadow: var(--shadow-focus);
}

.toolbar-search input {
  min-width: 0;
  flex: 1;
  border: 0;
  outline: 0;
  background: transparent;
  color: var(--text-primary);
}

.tunnel-toolbar select,
.sort-direction {
  padding: 0 var(--space-3);
}

.sort-direction {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: var(--space-2);
  cursor: pointer;
}

.tunnel-workspace {
  min-height: 0;
  display: grid;
  grid-template-columns: minmax(320px, 390px) minmax(0, 1fr);
  gap: var(--space-4);
}

.tunnel-list,
.tunnel-detail,
.detail-card {
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
}

.tunnel-list {
  min-height: 0;
  overflow: auto;
  padding: var(--space-2);
}

.tunnel-list__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-2);
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

.tunnel-list__header strong {
  color: var(--text-secondary);
  font-weight: var(--weight-semibold);
}

.tunnel-row {
  width: 100%;
  min-height: 72px;
  display: grid;
  grid-template-columns: 10px minmax(0, 1fr) auto;
  align-items: center;
  gap: var(--space-3);
  margin-top: var(--space-1);
  padding: var(--space-3);
  border: 1px solid transparent;
  border-radius: var(--radius-md);
  background: transparent;
  color: var(--text-primary);
  text-align: left;
  cursor: pointer;
}

.tunnel-row:hover,
.tunnel-row.active {
  border-color: var(--border-default);
  background: var(--bg-surface-hover);
}

.tunnel-row.active {
  box-shadow: inset 2px 0 0 var(--color-primary);
}

.tunnel-row__status,
.detail-title-row > span {
  width: 9px;
  height: 9px;
  border-radius: var(--radius-full);
  background: var(--status-offline);
}

.is-online {
  background: var(--status-online);
  color: var(--status-online);
}
.is-warning {
  background: var(--status-warning);
  color: var(--status-warning);
}
.is-error {
  background: var(--status-error);
  color: var(--status-error);
}
.is-offline {
  background: var(--status-offline);
  color: var(--status-offline);
}

.tunnel-row__main {
  min-width: 0;
}

.tunnel-row__main strong,
.tunnel-row__main small,
.tunnel-row__meta span,
.tunnel-row__meta small {
  display: block;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.tunnel-row__main small,
.tunnel-row__meta small {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

.tunnel-row__meta {
  color: var(--text-secondary);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  text-align: right;
}

.tunnel-list__empty,
.tunnel-detail__placeholder,
.tunnel-empty-state {
  display: grid;
  place-content: center;
  justify-items: center;
  gap: var(--space-3);
  color: var(--text-tertiary);
  text-align: center;
}

.tunnel-list__empty {
  min-height: 220px;
}

.tunnel-empty-state {
  min-height: 460px;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-xl);
  background: var(--bg-surface);
}

.empty-illustration {
  width: 86px;
  height: 86px;
  display: grid;
  place-items: center;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-2xl);
  background: var(--color-primary-muted);
  color: var(--color-primary);
}

.tunnel-empty-state h2 {
  color: var(--text-primary);
  font-size: var(--text-2xl);
  letter-spacing: 0;
}

.tunnel-empty-state p {
  max-width: 420px;
  color: var(--text-secondary);
}

.tunnel-detail {
  min-width: 0;
  min-height: 0;
  overflow: auto;
  padding: var(--space-5);
}

.detail-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-4);
}

.detail-title-row {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}

.detail-title-row h2 {
  font-size: var(--text-2xl);
  font-weight: var(--weight-semibold);
  letter-spacing: 0;
}

.detail-header p {
  margin-top: var(--space-1);
  color: var(--text-secondary);
}

.detail-actions {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  flex-shrink: 0;
}

.icon-action {
  width: 34px;
  height: 34px;
  display: grid;
  place-items: center;
  border: 1px solid var(--border-default);
  border-radius: var(--radius-md);
  background: var(--bg-input);
  color: var(--text-secondary);
  cursor: pointer;
}

.icon-action:hover,
.icon-action.active {
  color: var(--color-primary);
  border-color: var(--color-primary);
}

.detail-metrics {
  display: grid;
  grid-template-columns: repeat(4, minmax(0, 1fr));
  gap: var(--space-3);
  margin-top: var(--space-5);
}

.detail-metrics article {
  min-height: 82px;
  display: grid;
  align-content: center;
  gap: var(--space-1);
  padding: var(--space-3);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-input);
}

.detail-metrics span,
.path-list dt {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

.detail-metrics strong {
  min-width: 0;
  overflow: hidden;
  color: var(--text-primary);
  font-size: var(--text-lg);
  font-weight: var(--weight-semibold);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.test-url-panel {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  margin-top: var(--space-4);
  padding: var(--space-3);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-input);
}

.test-url-panel span,
.test-url-panel strong {
  display: block;
  min-width: 0;
}

.test-url-panel span {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
}

.test-url-panel strong {
  margin-top: 2px;
  overflow-wrap: anywhere;
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: var(--text-sm);
}

.test-url-panel__actions {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  flex-shrink: 0;
}

.detail-grid {
  display: grid;
  grid-template-columns: 1.1fr 0.9fr;
  gap: var(--space-4);
  margin-top: var(--space-4);
}

.detail-card {
  padding: var(--space-4);
}

.detail-card__heading {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
  margin-bottom: var(--space-3);
}

.detail-card__heading h3 {
  font-size: var(--text-md);
  font-weight: var(--weight-semibold);
}

.detail-card__heading button {
  width: 28px;
  height: 28px;
  display: grid;
  place-items: center;
  border: 0;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-secondary);
  cursor: pointer;
}

.detail-card__heading button:hover {
  background: var(--bg-surface-hover);
  color: var(--text-primary);
}

.path-list {
  display: grid;
  gap: var(--space-2);
}

.path-list div {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
}

.path-list dd {
  min-width: 0;
  overflow: hidden;
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.tag-list {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2);
}

.tag-list span {
  min-height: 24px;
  display: inline-flex;
  align-items: center;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-full);
  background: var(--bg-input);
  color: var(--text-secondary);
  padding: 0 var(--space-2);
  font-size: var(--text-xs);
}

.detail-card--logs {
  margin-top: var(--space-4);
}

.mini-log-list {
  display: grid;
  gap: var(--space-2);
}

.mini-log-list article {
  display: grid;
  grid-template-columns: 62px minmax(0, 1fr) auto;
  align-items: center;
  gap: var(--space-3);
  min-height: 34px;
  padding: 0 var(--space-2);
  border-radius: var(--radius-sm);
  background: var(--bg-input);
}

.mini-log-list span {
  background: transparent;
  font: var(--weight-semibold) var(--text-xs) var(--font-mono);
  text-transform: uppercase;
}

.mini-log-list span.is-info,
.mini-log-list span.is-success {
  color: var(--color-info);
}
.mini-log-list span.is-warn {
  color: var(--color-warning);
}
.mini-log-list span.is-error {
  color: var(--color-error);
}
.mini-log-list span.is-debug {
  color: var(--text-tertiary);
}

.mini-log-list p {
  min-width: 0;
  overflow: hidden;
  color: var(--text-secondary);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.mini-log-list small {
  color: var(--text-tertiary);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
}

.tunnel-detail__placeholder {
  min-height: 420px;
}

@media (max-width: 1120px) {
  .tunnel-workspace,
  .detail-grid {
    grid-template-columns: 1fr;
  }

  .tunnel-list {
    max-height: 360px;
  }

  .detail-metrics {
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }
}

@media (max-width: 760px) {
  .tunnels-hero,
  .detail-header {
    flex-direction: column;
  }

  .tunnels-hero__actions {
    width: 100%;
    align-items: stretch;
    flex-direction: column;
  }

  .tunnel-toolbar,
  .detail-metrics {
    grid-template-columns: 1fr;
  }

  .detail-actions {
    width: 100%;
    flex-wrap: wrap;
  }

  .test-url-panel {
    align-items: stretch;
    flex-direction: column;
  }

  .test-url-panel__actions {
    flex-wrap: wrap;
  }

  .mini-log-list article {
    grid-template-columns: 62px minmax(0, 1fr);
  }

  .mini-log-list small {
    display: none;
  }
}
</style>
