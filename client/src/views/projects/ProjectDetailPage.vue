<template>
  <section class="project-detail-page">
    <ProjectLoading v-if="store.status === 'loading'" :count="4" />

    <GCard v-else-if="store.status === 'error'" variant="plain" padding="lg">
      <GErrorState
        :title="t('project.detail.loadFailed')"
        :message="store.error || t('project.detail.loadFailedMessage')"
        retry
        @retry="store.load" />
    </GCard>

    <GCard v-else-if="!project" variant="plain" padding="lg">
      <GErrorState
        :title="t('project.detail.notFound')"
        :message="t('project.detail.notFoundMessage')">
        <template #action>
          <GButton variant="secondary" icon="arrow-left" @click="router.push('/projects')">
            {{ t('project.detail.backToProjects') }}
          </GButton>
        </template>
      </GErrorState>
    </GCard>

    <template v-else>
      <ProjectHeader
        :project="project"
        :loading="!!store.startingId || !!store.stoppingId"
        @back="router.push('/projects')"
        @edit="openEdit"
        @more="openDelete"
        @create-tunnel="createTunnel"
        @start-all="startAllWorkspace"
        @stop-all="stopAllWorkspace"
        @toggle-pin="togglePinProject"
        @toggle-favorite="toggleFavoriteProject" />

      <nav class="project-tabs" :aria-label="t('project.detail.tabsAria')">
        <button
          v-for="tab in tabs"
          :key="tab.key"
          type="button"
          class="project-tab"
          :class="{ active: activeTab === tab.key }"
          @click="activeTab = tab.key">
          <GIcon :name="tab.icon" :size="15" />
          <span>{{ tab.label }}</span>
        </button>
      </nav>

      <main class="project-tab-panel">
        <section v-if="activeTab === 'overview'" class="project-overview">
          <div class="project-stat-grid">
            <MetricCard
              :label="t('project.detail.runningTunnels')"
              :value="String(project.runningTunnelCount)"
              icon="router"
              tone="success" />
            <MetricCard
              label="HTTPS"
              :value="String(project.statistics.httpsCount)"
              icon="shield-check"
              tone="primary" />
            <MetricCard
              label="TCP"
              :value="String(project.statistics.tcpCount)"
              icon="network"
              tone="info" />
            <MetricCard
              :label="t('project.detail.traffic')"
              :value="formatBytes(project.statistics.totalTraffic)"
              icon="cloud"
              tone="warning" />
            <MetricCard
              label="CPU"
              :value="`${project.statistics.cpuUsage.toFixed(0)}%`"
              icon="cpu"
              tone="neutral" />
            <MetricCard
              :label="t('project.detail.memory')"
              :value="`${project.statistics.memoryUsage.toFixed(0)}%`"
              icon="memory-stick"
              tone="neutral" />
          </div>

          <section class="project-business-overview">
            <article>
              <span>关联 Tunnel</span>
              <strong>{{ project.tunnelCount }}</strong>
              <small>{{ project.runningTunnelCount }} running</small>
            </article>
            <article>
              <span>关联 Domain</span>
              <strong>{{ project.domainCount }}</strong>
              <small>{{ project.domains[0] || t('project.detail.unboundDomain') }}</small>
            </article>
            <article>
              <span>关联 Certificate</span>
              <strong>{{ project.certificateCount }}</strong>
              <small>{{ certificateLabel(project.certificateStatus) }}</small>
            </article>
            <article>
              <span>最近流量</span>
              <strong>{{ formatBytes(project.statistics.todayTraffic) }}</strong>
              <RuntimeSparkline :values="projectTrafficSparkline" label="Project traffic" />
            </article>
            <article>
              <span>最近请求</span>
              <strong>{{ project.statistics.requestCount }}</strong>
              <RuntimeSparkline :values="projectRequestSparkline" label="Project requests" />
            </article>
            <article>
              <span>运行状态</span>
              <strong>{{ project.status }}</strong>
              <small>{{ project.statistics.connections }} conn</small>
            </article>
            <article>
              <span>健康评分</span>
              <strong>{{ projectHealthScore }}</strong>
              <small>{{ project.statistics.errorCount }} errors</small>
            </article>
            <article>
              <span>启动时间</span>
              <strong>{{ formatDuration(project.statistics.uptime) }}</strong>
              <small>{{ project.lastStartedAt }}</small>
            </article>
            <article>
              <span>最后修改</span>
              <strong>{{ formatCompactTime(project.updatedAt) }}</strong>
              <small>{{ formatCompactTime(project.lastActivityAt) }}</small>
            </article>
          </section>

          <div class="project-detail-grid">
            <Panel :title="t('project.detail.recentLogs')" icon="logs">
              <LogList :logs="project.recentLogs" :empty="t('project.detail.noProjectLogs')" />
            </Panel>
            <Panel :title="t('project.detail.recentErrors')" icon="alert-triangle">
              <LogList :logs="project.recentErrors" :empty="t('project.detail.noRecentErrors')" />
            </Panel>
            <Panel :title="t('project.detail.certificateStatus')" icon="shield-check">
              <div class="certificate-state">
                <GBadge :variant="certificateVariant(project.certificateStatus)" type="soft">
                  {{ certificateLabel(project.certificateStatus) }}
                </GBadge>
                <span>{{
                  t('project.detail.certificateRefSummary', {
                    certificates: project.certificateCount,
                    domains: project.domainCount,
                  })
                }}</span>
              </div>
            </Panel>
          </div>
        </section>

        <section v-else-if="activeTab === 'tunnel'" class="resource-section">
          <div class="resource-toolbar">
            <select v-model="selectedTunnelId">
              <option value="">{{ t('project.detail.selectTunnel') }}</option>
              <option v-for="tunnel in availableTunnels" :key="tunnel.id" :value="tunnel.id">
                {{ tunnel.name }} · {{ tunnel.protocol.toUpperCase() }} · :{{ tunnel.remotePort }}
              </option>
            </select>
            <GButton
              variant="secondary"
              icon="link"
              :disabled="!selectedTunnelId"
              @click="addSelectedTunnel">
              {{ t('project.detail.addExistingTunnel') }}
            </GButton>
            <GButton variant="primary" icon="plus" @click="createTunnel">
              {{ t('project.detail.createNewTunnel') }}
            </GButton>
          </div>

          <div v-if="projectTunnels.length" class="resource-grid">
            <article v-for="tunnel in projectTunnels" :key="tunnel.id" class="resource-card">
              <div class="resource-card__head">
                <GBadge
                  :variant="tunnelStatusVariant(tunnel.status)"
                  type="soft">
                  {{ tunnel.status }}
                </GBadge>
                <GBadge variant="primary" type="outline" size="sm">
                  {{ tunnel.protocol.toUpperCase() }}
                </GBadge>
              </div>
              <h3>{{ tunnel.name }}</h3>
              <p>{{ tunnel.localHost }}:{{ tunnel.localPort }} → :{{ tunnel.remotePort }}</p>
              <div class="resource-card__metrics">
                <span>{{ tunnel.connections }} conn</span>
                <span>{{ formatBytes(tunnel.trafficBytes ?? 0) }}</span>
                <span>{{ tunnel.host || t('project.detail.unboundDomain') }}</span>
              </div>
              <div class="resource-card__actions">
                <select
                  :value="project.id"
                  :disabled="!otherProjects.length"
                  @change="moveTunnel(tunnel.id, readSelectValue($event))">
                  <option :value="project.id">{{ t('project.detail.moveTo') }}</option>
                  <option v-for="target in otherProjects" :key="target.id" :value="target.id">
                    {{ target.name }}
                  </option>
                </select>
                <GButton variant="ghost" size="sm" icon="unlink" @click="removeTunnel(tunnel.id)">
                  {{ t('project.detail.remove') }}
                </GButton>
              </div>
            </article>
          </div>

          <EmptyPanel
            v-else
            icon="router"
            :title="t('project.detail.noTunnels')"
            :message="t('project.detail.noTunnelsMessage')" />
        </section>

        <section v-else-if="activeTab === 'domain'" class="resource-section">
          <div class="resource-toolbar">
            <GInput v-model="domainInput" placeholder="example.com" prefix="globe" />
            <GButton
              variant="primary"
              icon="plus"
              :disabled="!domainInput.trim()"
              @click="addDomain">
              {{ t('project.detail.bindDomain') }}
            </GButton>
          </div>
          <div v-if="project.domains.length" class="resource-list">
            <article v-for="domain in project.domains" :key="domain" class="resource-row">
              <div>
                <strong>{{ domain }}</strong>
                <span>{{ domainResolution(domain) }}</span>
              </div>
              <GButton variant="ghost" size="sm" icon="unlink" @click="removeDomain(domain)">
                {{ t('project.detail.unbind') }}
              </GButton>
            </article>
          </div>
          <EmptyPanel
            v-else
            icon="globe"
            :title="t('project.detail.noDomains')"
            :message="t('project.detail.noDomainsMessage')" />
        </section>

        <section v-else-if="activeTab === 'certificate'" class="resource-section">
          <div class="resource-toolbar">
            <select v-model="selectedCertificateId">
              <option value="">{{ t('project.detail.selectCertificate') }}</option>
              <option
                v-for="certificate in store.certificates"
                :key="certificate.fingerprintSha256 || certificate.domain"
                :value="certificate.domain">
                {{ certificate.domain }} ·
                {{ t('project.detail.days', { count: certificate.daysRemaining }) }}
              </option>
            </select>
            <GButton
              variant="secondary"
              icon="link"
              :disabled="!selectedCertificateId"
              @click="addCertificate">
              {{ t('project.detail.referenceCertificate') }}
            </GButton>
            <GButton variant="primary" icon="external-link" @click="router.push('/certificates')">
              {{ t('project.detail.certificateCenter') }}
            </GButton>
          </div>
          <div v-if="projectCertificates.length" class="resource-list">
            <article
              v-for="certificate in projectCertificates"
              :key="certificate.fingerprintSha256 || certificate.domain"
              class="resource-row">
              <div>
                <strong>{{ certificate.domain }}</strong>
                <span>{{
                  `${certificate.issuer} · ${t('project.detail.expiresInDays', {
                    count: certificate.daysRemaining,
                  })}`
                }}</span>
              </div>
              <GButton
                variant="ghost"
                size="sm"
                icon="unlink"
                @click="removeCertificate(certificate.domain)">
                {{ t('project.detail.removeReference') }}
              </GButton>
            </article>
          </div>
          <EmptyPanel
            v-else
            icon="shield-check"
            :title="t('project.detail.noCertificates')"
            :message="t('project.detail.noCertificatesMessage')" />
        </section>

        <section v-else-if="activeTab === 'logs'" class="resource-section">
          <div class="resource-toolbar">
            <GSearchInput v-model="logQuery" :placeholder="t('project.detail.searchLogs')" />
            <select v-model="logLevel">
              <option value="all">{{ t('project.detail.allLevels') }}</option>
              <option value="info">{{ t('logs.level.info') }}</option>
              <option value="warn">{{ t('logs.level.warn') }}</option>
              <option value="error">{{ t('logs.level.error') }}</option>
              <option value="debug">{{ t('logs.level.debug') }}</option>
            </select>
            <select v-model="logWindow">
              <option value="all">{{ t('project.detail.allTimes') }}</option>
              <option value="hour">{{ t('project.detail.lastHour') }}</option>
              <option value="day">{{ t('project.detail.lastDay') }}</option>
            </select>
          </div>
          <LogList :logs="filteredLogs" :empty="t('project.detail.noMatchingLogs')" />
        </section>

        <section v-else-if="activeTab === 'metrics'" class="project-overview">
          <div class="project-stat-grid">
            <MetricCard
              :label="t('project.detail.connection')"
              :value="String(project.statistics.connections)"
              icon="link"
              tone="primary" />
            <MetricCard
              :label="t('project.detail.bandwidth')"
              :value="`${formatBytes(project.statistics.bandwidthBps)}/s`"
              icon="activity"
              tone="success" />
            <MetricCard
              :label="t('project.detail.request')"
              :value="String(project.statistics.requestCount)"
              icon="radio"
              tone="info" />
            <MetricCard
              label="TLS"
              :value="String(project.statistics.tlsSessionCount)"
              icon="shield-check"
              tone="primary" />
            <MetricCard
              :label="t('project.detail.error')"
              :value="String(project.statistics.errorCount)"
              icon="alert-triangle"
              tone="error" />
            <MetricCard
              :label="t('project.detail.latency')"
              :value="`${project.statistics.averageLatencyMs.toFixed(0)} ms`"
              icon="gauge"
              tone="warning" />
          </div>
        </section>

        <section v-else class="settings-grid">
          <GCard variant="plain" padding="md">
            <div class="settings-form">
              <label>
                <span>{{ t('project.detail.settings.name') }}</span>
                <GInput v-model="settingsForm.name" />
              </label>
              <label>
                <span>{{ t('project.detail.settings.description') }}</span>
                <GTextarea v-model="settingsForm.description" :rows="3" />
              </label>
              <label>
                <span>{{ t('project.detail.settings.icon') }}</span>
                <GInput v-model="settingsForm.icon" />
              </label>
              <label>
                <span>{{ t('project.detail.settings.color') }}</span>
                <select v-model="settingsForm.color">
                  <option v-for="color in colors" :key="color" :value="color">
                    {{ color }}
                  </option>
                </select>
              </label>
              <label>
                <span>{{ t('project.detail.settings.tags') }}</span>
                <GInput
                  v-model="settingsTags"
                  :placeholder="t('project.detail.settings.tagsPlaceholder')" />
              </label>
              <label>
                <span>{{ t('project.detail.settings.environments') }}</span>
                <GTextarea
                  v-model="environmentText"
                  :rows="5"
                  :placeholder="t('project.detail.settings.environmentsPlaceholder')" />
              </label>
              <label>
                <span>{{ t('project.detail.settings.startupPolicy') }}</span>
                <select v-model="settingsForm.startupPolicy">
                  <option :value="null">{{ t('project.detail.settings.manual') }}</option>
                  <option value="auto-start">
                    {{ t('project.detail.settings.autoStartReserved') }}
                  </option>
                </select>
              </label>
              <div class="settings-actions">
                <GButton variant="primary" icon="save" @click="saveSettings">
                  {{ t('project.detail.settings.save') }}
                </GButton>
              </div>
            </div>
          </GCard>
        </section>
      </main>

      <ProjectDialog
        v-model:visible="dialogVisible"
        :project="project"
        :server-names="store.serverNames"
        @submit="handleDialogSubmit" />

      <ProjectDeleteDialog
        v-model:visible="deleteVisible"
        :project="project"
        @confirm="handleDelete" />
    </template>
  </section>
</template>

<script setup lang="ts">
import { computed, defineComponent, h, onMounted, reactive, ref, watch, type PropType } from 'vue'
import { useI18n } from 'vue-i18n'
import { useRoute, useRouter } from 'vue-router'
import { useFeedback } from '@composables/useFeedback'
import GBadge from '@components/base/GBadge.vue'
import GButton from '@components/base/GButton.vue'
import GCard from '@components/base/GCard.vue'
import GIcon from '@components/icons/GIcon.vue'
import GErrorState from '@components/feedback/GErrorState.vue'
import GInput from '@components/form/GInput.vue'
import GSearchInput from '@components/form/GSearchInput.vue'
import GTextarea from '@components/form/GTextarea.vue'
import RuntimeSparkline from '@components/runtime/RuntimeSparkline.vue'
import { useMonitoringDashboard } from '@/monitoring/composables/useMonitoringDashboard'
import type { DashboardTunnel } from '@/monitoring/types'
import ProjectDeleteDialog from './components/ProjectDeleteDialog.vue'
import ProjectDialog from './components/ProjectDialog.vue'
import ProjectHeader from './components/ProjectHeader.vue'
import ProjectLoading from './components/ProjectLoading.vue'
import { projectTunnels as resolveProjectTunnels, useProjectStore } from './store/project'
import type {
  Project,
  ProjectColor,
  ProjectDeleteMode,
  ProjectFormData,
  ProjectLogEntry,
} from './types'
import { formatBytes } from './utils'
import './styles/project.css'

type ProjectTab = 'overview' | 'tunnel' | 'domain' | 'certificate' | 'logs' | 'metrics' | 'settings'

const route = useRoute()
const router = useRouter()
const store = useProjectStore()
const { t, locale } = useI18n()
const { toast, notify } = useFeedback()
const { metricHistory } = useMonitoringDashboard()

const activeTab = ref<ProjectTab>('overview')
const selectedTunnelId = ref('')
const selectedCertificateId = ref('')
const domainInput = ref('')
const logQuery = ref('')
const logLevel = ref('all')
const logWindow = ref('all')
const dialogVisible = ref(false)
const deleteVisible = ref(false)
const settingsTags = ref('')
const environmentText = ref('')
const settingsForm = reactive({
  name: '',
  description: '',
  icon: 'package',
  color: 'blue' as ProjectColor,
  startupPolicy: null as string | null,
})

const tabs = computed<Array<{ key: ProjectTab; label: string; icon: string }>>(() => [
  { key: 'overview', label: t('project.detail.tabs.overview'), icon: 'dashboard' },
  { key: 'tunnel', label: t('project.detail.tabs.tunnel'), icon: 'router' },
  { key: 'domain', label: t('project.detail.tabs.domain'), icon: 'globe' },
  { key: 'certificate', label: t('project.detail.tabs.certificate'), icon: 'shield-check' },
  { key: 'logs', label: t('project.detail.tabs.logs'), icon: 'logs' },
  { key: 'metrics', label: t('project.detail.tabs.metrics'), icon: 'chart-line' },
  { key: 'settings', label: t('project.detail.tabs.settings'), icon: 'settings' },
])
const colors: ProjectColor[] = [
  'blue',
  'green',
  'purple',
  'orange',
  'red',
  'cyan',
  'pink',
  'indigo',
  'teal',
  'amber',
  'slate',
]

const project = computed(() => store.getById(String(route.params.projectId)))
const projectTunnels = computed(() =>
  project.value ? resolveProjectTunnels(project.value, store.dashboard) : [],
)
const projectTunnelIds = computed(() => new Set(project.value?.tunnelIds ?? []))
const availableTunnels = computed(() =>
  (store.dashboard?.tunnels ?? []).filter((tunnel) => !projectTunnelIds.value.has(tunnel.id)),
)
const otherProjects = computed(() => store.projects.filter((item) => item.id !== project.value?.id))
const projectCertificates = computed(() => {
  if (!project.value) return []
  return store.certificates.filter(
    (certificate) =>
      project.value?.certificateIds.includes(certificate.domain) ||
      project.value?.certificateIds.includes(certificate.fingerprintSha256) ||
      project.value?.domains.includes(certificate.domain),
  )
})
const projectTrafficSparkline = computed(() =>
  metricHistory.value.length
    ? metricHistory.value.map((point) => point.uploadBps + point.downloadBps)
    : [project.value?.statistics.bandwidthBps ?? 0],
)
const projectRequestSparkline = computed(() =>
  metricHistory.value.length
    ? metricHistory.value.map((point) => point.requests)
    : [project.value?.statistics.requestCount ?? 0],
)
const projectHealthScore = computed(() => {
  const current = project.value
  if (!current) return '0/100'
  const tunnelScore = current.tunnelCount
    ? (current.runningTunnelCount / current.tunnelCount) * 70
    : 70
  const errorPenalty = Math.min(30, current.statistics.errorCount * 2)
  return `${Math.max(0, Math.round(tunnelScore + 30 - errorPenalty))}/100`
})

function tunnelStatusVariant(
  status: DashboardTunnel['status'],
): 'success' | 'error' | 'neutral' {
  if (isRunningTunnelStatus(status)) return 'success'
  if (status === 'warning' || status === 'error') return 'error'
  return 'neutral'
}

function isRunningTunnelStatus(status: DashboardTunnel['status']): boolean {
  return ['running', 'starting', 'restarting', 'recovering'].includes(status)
}

const filteredLogs = computed(() => {
  const current = project.value
  if (!current) return []
  const keyword = logQuery.value.trim().toLowerCase()
  const since =
    logWindow.value === 'hour'
      ? Date.now() - 60 * 60 * 1000
      : logWindow.value === 'day'
        ? Date.now() - 24 * 60 * 60 * 1000
        : 0

  return current.recentLogs.filter((log) => {
    if (logLevel.value !== 'all' && log.level !== logLevel.value) return false
    if (since && log.timestamp < since) return false
    if (!keyword) return true
    return [log.level, log.source, log.message].join(' ').toLowerCase().includes(keyword)
  })
})

onMounted(() => {
  if (store.status === 'idle') {
    void store.load()
  }
})

watch(
  project,
  (value) => {
    if (!value) return
    settingsForm.name = value.name
    settingsForm.description = value.description
    settingsForm.icon = value.icon
    settingsForm.color = value.color
    settingsForm.startupPolicy = value.startupPolicy ?? null
    settingsTags.value = value.tags.join(', ')
    environmentText.value = environmentToText(value)
  },
  { immediate: true },
)

function openEdit() {
  dialogVisible.value = true
}

function openDelete() {
  deleteVisible.value = true
}

function createTunnel() {
  void router.push({ path: '/tunnels', query: { create: '1', projectId: project.value?.id } })
}

async function addSelectedTunnel() {
  if (!project.value || !selectedTunnelId.value) return
  try {
    await store.addTunnel(project.value.id, selectedTunnelId.value)
    selectedTunnelId.value = ''
    toast.success(t('project.notifications.tunnelAdded'))
  } catch (err) {
    notify.error(t('project.notifications.tunnelAddFailed'), errorMessage(err), 10000)
  }
}

async function removeTunnel(tunnelId: string) {
  if (!project.value) return
  try {
    await store.removeTunnel(project.value.id, tunnelId)
    toast.success(t('project.notifications.tunnelRemoved'))
  } catch (err) {
    notify.error(t('project.notifications.tunnelRemoveFailed'), errorMessage(err), 10000)
  }
}

async function moveTunnel(tunnelId: string, targetProjectId: string) {
  if (!project.value || !targetProjectId || targetProjectId === project.value.id) return
  try {
    await store.moveTunnel(project.value.id, targetProjectId, tunnelId)
    toast.success(t('project.notifications.tunnelMoved'))
  } catch (err) {
    notify.error(t('project.notifications.tunnelMoveFailed'), errorMessage(err), 10000)
  }
}

async function addDomain() {
  if (!project.value || !domainInput.value.trim()) return
  try {
    await store.addDomain(project.value.id, domainInput.value.trim())
    domainInput.value = ''
    toast.success(t('project.notifications.domainBound'))
  } catch (err) {
    notify.error(t('project.notifications.domainBindFailed'), errorMessage(err), 10000)
  }
}

async function removeDomain(domain: string) {
  if (!project.value) return
  try {
    await store.removeDomain(project.value.id, domain)
    toast.success(t('project.notifications.domainUnbound'))
  } catch (err) {
    notify.error(t('project.notifications.domainUnbindFailed'), errorMessage(err), 10000)
  }
}

async function addCertificate() {
  if (!project.value || !selectedCertificateId.value) return
  try {
    await store.addCertificate(project.value.id, selectedCertificateId.value)
    selectedCertificateId.value = ''
    toast.success(t('project.notifications.certificateReferenced'))
  } catch (err) {
    notify.error(t('project.notifications.certificateReferenceFailed'), errorMessage(err), 10000)
  }
}

async function removeCertificate(certificateId: string) {
  if (!project.value) return
  try {
    await store.removeCertificate(project.value.id, certificateId)
    toast.success(t('project.notifications.certificateReferenceRemoved'))
  } catch (err) {
    notify.error(
      t('project.notifications.certificateReferenceRemoveFailed'),
      errorMessage(err),
      10000,
    )
  }
}

async function saveSettings() {
  if (!project.value) return
  try {
    const environments = environmentText.value.trim()
      ? [
          {
            id: project.value.environments[0]?.id ?? crypto.randomUUID(),
            name: t('project.detail.defaultEnvironment'),
            variables: environmentText.value
              .split('\n')
              .map((line) => line.trim())
              .filter(Boolean)
              .map((line) => {
                const [key, ...rest] = line.split('=')
                return {
                  key: key.trim(),
                  value: rest.join('=').trim(),
                  secret: false,
                }
              })
              .filter((item) => item.key),
          },
        ]
      : []

    await store.updateProject(project.value.id, {
      name: settingsForm.name,
      description: settingsForm.description,
      icon: settingsForm.icon,
      color: settingsForm.color,
      tags: settingsTags.value
        .split(',')
        .map((tag) => tag.trim())
        .filter(Boolean),
      environments,
      startupPolicy: settingsForm.startupPolicy,
    })
    toast.success(t('project.notifications.settingsSaved'))
  } catch (err) {
    notify.error(t('project.notifications.settingsSaveFailed'), errorMessage(err), 10000)
  }
}

async function handleDialogSubmit(form: ProjectFormData) {
  if (!project.value) return
  try {
    await store.updateProject(project.value.id, form)
    toast.success(t('project.notifications.updated', { name: form.name }))
  } catch (err) {
    notify.error(t('project.notifications.saveFailed'), errorMessage(err), 10000)
  }
}

async function handleDelete(target: Project, mode: ProjectDeleteMode) {
  try {
    await store.removeProject(target.id, mode)
    toast.success(t('project.notifications.deleted', { name: target.name }))
    void router.push('/projects')
  } catch (err) {
    notify.error(t('project.notifications.deleteFailed'), errorMessage(err), 10000)
  }
}

async function togglePinProject(id: string) {
  try {
    await store.togglePin(id)
  } catch (err) {
    notify.error(t('project.notifications.pinFailed'), errorMessage(err), 8000)
  }
}

async function toggleFavoriteProject(id: string) {
  try {
    await store.toggleFavorite(id)
  } catch (err) {
    notify.error(t('project.notifications.favoriteFailed'), errorMessage(err), 8000)
  }
}

async function startAllWorkspace() {
  if (!project.value || store.startingId) return
  try {
    await store.startProject(project.value.id)
    toast.success(t('project.notifications.started', { name: project.value.name }))
  } catch (err) {
    notify.error(t('project.notifications.startFailed'), errorMessage(err), 8000)
  }
}

async function stopAllWorkspace() {
  if (!project.value || store.stoppingId) return
  try {
    await store.stopProject(project.value.id)
    toast.warning(t('project.notifications.stopped', { name: project.value.name }))
  } catch (err) {
    notify.error(t('project.notifications.stopFailed'), errorMessage(err), 8000)
  }
}

function domainResolution(domain: string) {
  return projectTunnels.value.some((tunnel) => tunnel.host === domain)
    ? t('project.detail.domainLinked')
    : t('project.detail.domainPending')
}

function certificateVariant(status: Project['certificateStatus']) {
  if (status === 'healthy') return 'success'
  if (status === 'warning') return 'warning'
  if (status === 'missing') return 'error'
  return 'neutral'
}

function certificateLabel(status: Project['certificateStatus']) {
  return t(`project.detail.certificateLabels.${status}`)
}

function environmentToText(value: Project) {
  return value.environments
    .flatMap((environment) => environment.variables)
    .map((item) => `${item.key}=${item.value}`)
    .join('\n')
}

function formatLogTime(timestamp: number) {
  return new Intl.DateTimeFormat(locale.value === 'en-US' ? 'en-US' : 'zh-CN', {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  }).format(timestamp)
}

function formatCompactTime(timestamp: number) {
  if (!Number.isFinite(timestamp) || timestamp <= 0) return '-'
  return new Intl.DateTimeFormat(locale.value === 'en-US' ? 'en-US' : 'zh-CN', {
    month: '2-digit',
    day: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  }).format(timestamp)
}

function formatDuration(seconds: number): string {
  if (!Number.isFinite(seconds) || seconds <= 0) return '-'
  const day = Math.floor(seconds / 86400)
  const hour = Math.floor((seconds % 86400) / 3600)
  const minute = Math.floor((seconds % 3600) / 60)
  if (day) return `${day}d ${hour}h`
  if (hour) return `${hour}h ${minute}m`
  return `${Math.max(1, minute)}m`
}

function readSelectValue(event: Event) {
  return (event.target as HTMLSelectElement).value
}

function errorMessage(err: unknown): string {
  if (typeof err === 'string') return err
  if (err instanceof Error && err.message) return err.message
  return t('project.notifications.storageCheck')
}

const MetricCard = defineComponent({
  props: {
    label: { type: String, required: true },
    value: { type: String, required: true },
    icon: { type: String, required: true },
    tone: { type: String, default: 'neutral' },
  },
  setup(props) {
    return () =>
      h('article', { class: ['metric-card', `metric-card--${props.tone}`] }, [
        h('span', { class: 'metric-card__icon' }, [h(GIcon, { name: props.icon, size: 18 })]),
        h('strong', props.value),
        h('small', props.label),
      ])
  },
})

const Panel = defineComponent({
  props: {
    title: { type: String, required: true },
    icon: { type: String, required: true },
  },
  setup(props, { slots }) {
    return () =>
      h(
        GCard,
        { variant: 'plain', padding: 'md', class: 'project-panel' },
        {
          default: () => [
            h('div', { class: 'project-panel__head' }, [
              h('span', [h(GIcon, { name: props.icon, size: 16 }), props.title]),
            ]),
            slots.default?.(),
          ],
        },
      )
  },
})

const LogList = defineComponent({
  props: {
    logs: { type: Array as PropType<ProjectLogEntry[]>, required: true },
    empty: { type: String, required: true },
  },
  setup(props) {
    return () =>
      props.logs.length
        ? h(
            'div',
            { class: 'project-log-list' },
            props.logs.map((log) =>
              h('article', { key: log.id, class: 'project-log-row' }, [
                h('span', { class: `is-${log.level}` }, log.level),
                h('p', log.message),
                h('small', formatLogTime(log.timestamp)),
              ]),
            ),
          )
        : h('div', { class: 'project-mini-empty' }, props.empty)
  },
})

const EmptyPanel = defineComponent({
  props: {
    icon: { type: String, required: true },
    title: { type: String, required: true },
    message: { type: String, required: true },
  },
  setup(props) {
    return () =>
      h('div', { class: 'project-empty-panel' }, [
        h(GIcon, { name: props.icon, size: 30 }),
        h('strong', props.title),
        h('span', props.message),
      ])
  },
})
</script>

<style scoped>
.project-tabs {
  display: flex;
  align-items: center;
  gap: var(--space-1);
  overflow-x: auto;
  padding: 3px;
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-input);
}

.project-tab {
  min-width: max-content;
  height: 32px;
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  padding: 0 var(--space-3);
  border: 0;
  border-radius: var(--radius-sm);
  background: transparent;
  color: var(--text-tertiary);
  cursor: pointer;
}

.project-tab:hover,
.project-tab.active {
  background: var(--bg-surface-hover);
  color: var(--text-primary);
}

.project-tab-panel {
  min-height: 0;
  overflow: auto;
}

.project-overview,
.resource-section,
.settings-grid {
  display: grid;
  gap: var(--space-4);
}

.metric-card {
  min-height: 104px;
  display: grid;
  align-content: center;
  gap: var(--space-1);
  padding: var(--space-4);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-card);
}

.metric-card__icon {
  width: 36px;
  height: 36px;
  display: grid;
  place-items: center;
  margin-bottom: var(--space-2);
  border-radius: var(--radius-md);
  background: var(--bg-input);
  color: var(--color-primary);
}

.metric-card strong {
  color: var(--text-primary);
  font-size: var(--text-xl);
  font-weight: var(--weight-semibold);
  font-variant-numeric: tabular-nums;
  overflow-wrap: anywhere;
}

.metric-card small {
  color: var(--text-tertiary);
}

.metric-card--success .metric-card__icon {
  color: var(--color-success);
  background: var(--color-success-muted);
}
.metric-card--warning .metric-card__icon {
  color: var(--color-warning);
  background: var(--color-warning-muted);
}
.metric-card--error .metric-card__icon {
  color: var(--color-error);
  background: var(--color-error-muted);
}
.metric-card--info .metric-card__icon {
  color: var(--color-info);
  background: var(--color-info-muted);
}

.project-business-overview {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: var(--space-3);
}

.project-business-overview article {
  min-width: 0;
  min-height: 94px;
  display: grid;
  align-content: center;
  gap: var(--space-1);
  padding: var(--space-3);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-input);
}

.project-business-overview span,
.project-business-overview small {
  min-width: 0;
  overflow: hidden;
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.project-business-overview strong {
  min-width: 0;
  overflow: hidden;
  color: var(--text-primary);
  font-family: var(--font-mono);
  font-size: var(--text-lg);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.project-detail-grid {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--space-4);
}

.project-panel__head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: var(--space-3);
}

.project-panel__head span {
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  color: var(--text-primary);
  font-weight: var(--weight-semibold);
}

.project-log-list {
  display: grid;
  gap: var(--space-2);
}

.project-log-row {
  min-height: 34px;
  display: grid;
  grid-template-columns: 62px minmax(0, 1fr) auto;
  align-items: center;
  gap: var(--space-3);
  padding: 0 var(--space-2);
  border-radius: var(--radius-sm);
  background: var(--bg-input);
}

.project-log-row span {
  font: var(--weight-semibold) var(--text-xs) var(--font-mono);
  text-transform: uppercase;
}

.project-log-row span.is-info,
.project-log-row span.is-success {
  color: var(--color-info);
}
.project-log-row span.is-warn {
  color: var(--color-warning);
}
.project-log-row span.is-error {
  color: var(--color-error);
}
.project-log-row span.is-debug {
  color: var(--text-tertiary);
}

.project-log-row p {
  min-width: 0;
  overflow: hidden;
  color: var(--text-secondary);
  text-overflow: ellipsis;
  white-space: nowrap;
}

.project-log-row small {
  color: var(--text-tertiary);
  font-family: var(--font-mono);
}

.project-mini-empty,
.project-empty-panel {
  min-height: 160px;
  display: grid;
  place-content: center;
  justify-items: center;
  gap: var(--space-2);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
  color: var(--text-tertiary);
  text-align: center;
}

.project-empty-panel strong {
  color: var(--text-primary);
}

.project-empty-panel span {
  max-width: 420px;
  line-height: var(--leading-relaxed);
}

.certificate-state {
  min-height: 120px;
  display: grid;
  align-content: center;
  gap: var(--space-2);
  color: var(--text-secondary);
}

.resource-toolbar {
  display: grid;
  grid-template-columns: minmax(220px, 1fr) auto auto;
  gap: var(--space-2);
}

.resource-toolbar select,
.resource-card__actions select,
.settings-form select {
  min-width: 0;
  height: var(--control-height-md);
  border: 1px solid var(--color-border);
  border-radius: var(--radius-input);
  background: var(--bg-input);
  color: var(--text-primary);
  padding: 0 var(--space-3);
}

.resource-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: var(--space-3);
}

.resource-card,
.resource-row {
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-card);
}

.resource-card {
  display: grid;
  gap: var(--space-3);
  padding: var(--space-4);
}

.resource-card__head,
.resource-card__actions,
.resource-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: var(--space-3);
}

.resource-card h3 {
  color: var(--text-primary);
  font-size: var(--text-lg);
  font-weight: var(--weight-semibold);
}

.resource-card p,
.resource-row span {
  color: var(--text-tertiary);
  font-family: var(--font-mono);
  font-size: var(--text-xs);
}

.resource-card__metrics {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2);
  color: var(--text-secondary);
  font-size: var(--text-xs);
}

.resource-list {
  display: grid;
  gap: var(--space-2);
}

.resource-row {
  min-height: 64px;
  padding: var(--space-3);
}

.resource-row div {
  display: grid;
  gap: 2px;
}

.resource-row strong {
  color: var(--text-primary);
}

.settings-form {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--space-4);
}

.settings-form label {
  display: grid;
  gap: var(--space-2);
}

.settings-form label:nth-child(2),
.settings-form label:nth-child(6),
.settings-actions {
  grid-column: 1 / -1;
}

.settings-form label span {
  color: var(--text-secondary);
  font-size: var(--text-sm);
  font-weight: var(--weight-medium);
}

.settings-actions {
  display: flex;
  justify-content: flex-end;
}

@media (max-width: 960px) {
  .project-detail-grid,
  .project-business-overview,
  .settings-form,
  .resource-toolbar {
    grid-template-columns: 1fr;
  }
}
</style>
