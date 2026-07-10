<template>
  <section class="help-center-page">
    <header class="help-hero">
      <div>
        <p>{{ t('help.hero.kicker') }}</p>
        <h1>{{ t('help.hero.title') }}</h1>
        <span>{{ t('help.hero.description') }}</span>
      </div>
      <div class="help-hero__actions">
        <div v-if="diagnosticRunMessage" class="diagnostic-run-state" :class="diagnosticRunTone">
          <GIcon :name="loading ? 'loader' : diagnosticRunIcon" :size="14" :spin="loading" />
          <span>{{ diagnosticRunMessage }}</span>
        </div>
        <GButton variant="primary" icon="activity" :loading="loading" @click="refreshDiagnostics()">
          {{ loading ? t('help.actions.running') : t('help.actions.start') }}
        </GButton>
        <GButton variant="secondary" icon="copy" @click="copyDiagnosticReport()">
          {{ t('help.actions.copyReport') }}
        </GButton>
      </div>
    </header>

    <section class="help-section" :aria-label="t('help.sections.systemStatusKicker')">
      <header class="section-heading">
        <div>
          <p>{{ t('help.sections.systemStatusKicker') }}</p>
          <h2>{{ t('help.sections.systemStatus') }}</h2>
        </div>
      </header>

      <div class="status-grid">
        <StatusCard
          v-for="item in statusCards"
          :key="item.title"
          :title="item.title"
          :value="item.value"
          :detail="item.detail"
          :status="item.status"
          :icon="item.icon"
          :loading="loading && item.status === 'unknown'" />
      </div>
    </section>

    <div class="help-layout">
      <main class="help-main">
        <section class="help-section">
          <header class="section-heading">
            <div>
              <p>{{ t('help.sections.checklistKicker') }}</p>
              <h2>{{ t('help.sections.checklist') }}</h2>
            </div>
            <span>{{ checklistSummary }}</span>
          </header>

          <div class="diagnostic-list">
            <DiagnosticCard
              v-for="item in diagnosticChecklist"
              :key="item.id"
              :title="item.title"
              :description="item.description"
              :status="item.status"
              :reason="item.reason"
              :solution="item.solution"
              :meta="item.meta" />
          </div>
        </section>

        <section v-if="deploymentFindings.length" class="help-section">
          <header class="section-heading">
            <div>
              <p>{{ t('help.sections.deploymentKicker') }}</p>
              <h2>{{ t('help.sections.deployment') }}</h2>
            </div>
            <span>{{ deploymentSummary }}</span>
          </header>

          <div class="finding-strip">
            <article
              v-for="finding in localizedDeploymentFindings"
              :key="finding.id"
              :class="`is-${finding.status}`">
              <GIcon :name="findingIcon(finding.status)" :size="16" />
              <div>
                <strong>{{ finding.label }}</strong>
                <p>{{ finding.reason }}</p>
              </div>
            </article>
          </div>
        </section>

        <section class="help-section">
          <header class="section-heading">
            <div>
              <p>{{ t('help.sections.reportKicker') }}</p>
              <h2>{{ t('help.sections.report') }}</h2>
            </div>
            <span>{{ reportFreshness }}</span>
          </header>

          <div class="report-grid">
            <ReportCard
              :title="t('help.report.copy.title')"
              :description="t('help.report.copy.description')"
              icon="copy"
              tone="primary"
              @action="copyDiagnosticReport" />
            <ReportCard
              :title="t('help.report.bundle.title')"
              :description="t('help.report.bundle.description')"
              icon="package"
              @action="exportDebugBundle" />
            <ReportCard
              :title="t('help.report.issue.title')"
              :description="t('help.report.issue.description')"
              icon="github"
              @action="openIssue" />
          </div>

          <SystemInfoCard
            :title="t('help.report.contentsTitle')"
            :description="t('help.report.contentsDescription')"
            :rows="reportRows" />

          <div class="recent-errors">
            <header>
              <strong>{{ t('help.recentErrors.title') }}</strong>
              <span>{{ t('help.units.entries', { count: recentErrors.length }) }}</span>
            </header>
            <article v-for="error in recentErrors" :key="error.id">
              <span>{{ error.source }}</span>
              <div>
                <strong>{{ error.title }}</strong>
                <p>{{ error.detail }}</p>
              </div>
            </article>
            <p v-if="!recentErrors.length" class="empty-copy">
              {{ t('help.recentErrors.empty') }}
            </p>
          </div>
        </section>
      </main>

      <aside class="help-side">
        <SystemInfoCard
          :title="t('help.side.systemInfo')"
          :description="t('help.side.systemInfoDescription')"
          :rows="systemRows">
          <template #actions>
            <GButton variant="ghost" size="sm" icon="copy" @click="copySystemInfo" />
          </template>
        </SystemInfoCard>

        <SystemInfoCard
          :title="t('help.side.runtimeState')"
          :description="t('help.side.runtimeStateDescription')"
          :rows="runtimeRows" />

        <section class="advanced-tools">
          <header class="section-heading compact">
            <div>
              <p>{{ t('help.sections.toolsKicker') }}</p>
              <h2>{{ t('help.sections.tools') }}</h2>
            </div>
          </header>

          <div class="tool-grid">
            <ReportCard
              :title="t('help.tools.openLogs.title')"
              :description="systemInfo?.logDir ?? t('help.states.waitingSystemInfo')"
              icon="logs"
              @action="openLogDirectory" />
            <ReportCard
              :title="t('help.tools.openConfig.title')"
              :description="systemInfo?.configDir ?? t('help.states.waitingSystemInfo')"
              icon="settings"
              @action="openConfigDirectory" />
            <ReportCard
              :title="t('help.tools.refresh.title')"
              :description="t('help.tools.refresh.description')"
              icon="refresh"
              @action="refreshDiagnostics" />
            <ReportCard
              :title="t('help.tools.clearCache.title')"
              :description="t('help.tools.clearCache.description')"
              icon="trash"
              tone="danger"
              @action="clearSupportCache" />
            <ReportCard
              :title="t('help.tools.exportLogs.title')"
              :description="t('help.tools.exportLogs.description')"
              icon="download"
              @action="exportLogs" />
          </div>
        </section>
      </aside>
    </div>
  </section>
</template>

<script setup lang="ts">
import { computed, onMounted, reactive, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { open as openExternal } from '@tauri-apps/plugin-shell'
import GButton from '@components/base/GButton.vue'
import GIcon from '@components/icons/GIcon.vue'
import { useFeedback } from '@composables/useFeedback'
import { TauriIpcClient } from '@/ipc'
import { GITHUB_ISSUE_URL } from '@/constants'
import {
  DIAGNOSTIC_VALUE_DISCONNECTED,
  DIAGNOSTIC_VALUE_MEMORY_PERMISSION_REQUIRED,
  diagnosticsService,
  serverService,
} from '@/services'
import type {
  ConnectionHistoryEntry,
  ConnectionTestReport,
  DiagnosticFinding,
  DeploymentCheckReport,
  DiagnosticStatus,
  RecentServer,
  RuntimeServerList,
  RuntimeServerRecord,
  SystemInfoReport,
} from '@/services'
import type { CertificateListResponse, CertificateStatus } from '@views/certificates/types'
import { certificateService } from '@views/certificates/service'
import type { DashboardData, DashboardTunnel, HealthReport, HealthStatus } from '@/monitoring/types'
import { useLogStore } from '@views/logs/store'
import { downloadLogs } from '@views/logs/utils'
import DiagnosticCard from './components/DiagnosticCard.vue'
import ReportCard from './components/ReportCard.vue'
import StatusCard from './components/StatusCard.vue'
import SystemInfoCard from './components/SystemInfoCard.vue'

type SupportStatus = 'ok' | 'warning' | 'error' | 'unknown'
type ChecklistStatus = 'ok' | 'warning' | 'error'
type DiagnosticScope =
  | 'system'
  | 'servers'
  | 'runtime'
  | 'health'
  | 'deployment'
  | 'connection'
  | 'certificates'
  | 'logs'

interface StatusItem {
  title: string
  value: string
  detail: string
  status: SupportStatus
  icon: string
}

interface ChecklistItem {
  id: string
  title: string
  description: string
  status: ChecklistStatus
  reason: string
  solution: string
  meta?: string
}

interface RecentErrorItem {
  id: string
  source: string
  title: string
  detail: string
}

interface DiagnosticRunMessage {
  key: string
  params?: Record<string, unknown>
}

interface DiagnosticRunSummary extends DiagnosticRunMessage {
  tone: 'success' | 'warning' | 'error'
}

const ipc = new TauriIpcClient()
const logStore = useLogStore()
const { toast } = useFeedback()
const { t, te } = useI18n()

const loading = ref(false)
const lastGeneratedAt = ref(0)
const diagnosticRunMessageKey = ref<DiagnosticRunMessage | null>(null)
const diagnosticRunTone = ref<'info' | 'success' | 'warning' | 'error'>('info')
const systemInfo = ref<SystemInfoReport | null>(null)
const serverList = ref<RuntimeServerList | null>(null)
const dashboard = ref<DashboardData | null>(null)
const health = ref<HealthReport | null>(null)
const certificates = ref<CertificateListResponse | null>(null)
const deploymentReport = ref<DeploymentCheckReport | null>(null)
const connectionReport = ref<ConnectionTestReport | null>(null)
const recentServers = ref<RecentServer[]>([])
const connectionHistory = ref<ConnectionHistoryEntry[]>([])

const errors = reactive<Record<DiagnosticScope, string>>({
  system: '',
  servers: '',
  runtime: '',
  health: '',
  deployment: '',
  connection: '',
  certificates: '',
  logs: '',
})

const selectedServer = computed(() => {
  const list = serverList.value
  if (!list?.items.length) return null
  return (
    list.items.find((server) => server.id === list.activeServerId) ??
    list.items.find((server) => server.status === 'connected') ??
    list.items[0]
  )
})

const selectedServerAddr = computed(() => {
  const server = selectedServer.value
  return server ? `${server.host}:${server.port}` : undefined
})

const statusCards = computed<StatusItem[]>(() => [
  clientStatus.value,
  serverStatus.value,
  runtimeStatus.value,
  tunnelStatus.value,
  certificateStatus.value,
])

const diagnosticRunIcon = computed(() => {
  if (diagnosticRunTone.value === 'success') return 'check-circle'
  if (diagnosticRunTone.value === 'warning') return 'alert-triangle'
  if (diagnosticRunTone.value === 'error') return 'alert-circle'
  return 'info-circle'
})

const diagnosticRunMessage = computed(() =>
  diagnosticRunMessageKey.value
    ? t(diagnosticRunMessageKey.value.key, diagnosticRunMessageKey.value.params ?? {})
    : '',
)

const clientStatus = computed<StatusItem>(() => {
  if (systemInfo.value) {
    return {
      title: t('help.status.client.title'),
      value: `v${systemInfo.value.clientVersion}`,
      detail: `${systemInfo.value.os} / ${systemInfo.value.arch}`,
      status: 'ok',
      icon: 'monitor',
    }
  }
  return {
    title: t('help.status.client.title'),
    value: errors.system ? t('help.states.unavailable') : t('help.states.waitingCheck'),
    detail: errors.system || t('help.status.client.waitingDetail'),
    status: errors.system ? 'error' : 'unknown',
    icon: 'monitor',
  }
})

const serverStatus = computed<StatusItem>(() => {
  const list = serverList.value
  if (!list) {
    return {
      title: t('help.status.server.title'),
      value: errors.servers ? t('help.states.unavailable') : t('help.states.waitingCheck'),
      detail: errors.servers || t('help.status.server.waitingDetail'),
      status: errors.servers ? 'error' : 'unknown',
      icon: 'servers',
    }
  }
  const active = selectedServer.value
  if (list.connected && active) {
    return {
      title: t('help.status.server.title'),
      value: t('help.states.connected'),
      detail: `${active.name || active.host} · ${active.host}:${active.port}`,
      status: 'ok',
      icon: 'servers',
    }
  }
  if (list.items.length) {
    return {
      title: t('help.status.server.title'),
      value: t('help.states.disconnected'),
      detail: t('help.status.server.savedButDisconnected', { count: list.items.length }),
      status: 'warning',
      icon: 'servers',
    }
  }
  return {
    title: t('help.status.server.title'),
    value: t('help.states.notConfigured'),
    detail: t('help.status.server.notConfiguredDetail'),
    status: 'warning',
    icon: 'servers',
  }
})

const runtimeStatus = computed<StatusItem>(() => {
  const runtimeError = errors.runtime || errors.health
  if (runtimeError) {
    return {
      title: t('help.status.runtime.title'),
      value: t('help.states.unavailable'),
      detail: runtimeError,
      status: 'error',
      icon: 'cpu',
    }
  }
  if (!health.value) {
    return {
      title: t('help.status.runtime.title'),
      value: t('help.states.waitingCheck'),
      detail: t('help.status.runtime.waitingDetail'),
      status: 'unknown',
      icon: 'cpu',
    }
  }
  return {
    title: t('help.status.runtime.title'),
    value: healthLabel(health.value.overall),
    detail: t('help.status.runtime.signalDetail', {
      count: health.value.signals.length,
      time: formatTime(health.value.updatedAt),
    }),
    status: mapHealthStatus(health.value.overall),
    icon: 'cpu',
  }
})

const tunnelStatus = computed<StatusItem>(() => {
  const data = dashboard.value
  if (!data) {
    return {
      title: t('help.status.tunnel.title'),
      value: errors.runtime ? t('help.states.unavailable') : t('help.states.waitingCheck'),
      detail: errors.runtime || t('help.status.tunnel.waitingDetail'),
      status: errors.runtime ? 'error' : 'unknown',
      icon: 'router',
    }
  }
  const total = data.tunnels.length
  const running = data.tunnels.filter((tunnel) => isRunningTunnelStatus(tunnel.status)).length
  if (running > 0) {
    return {
      title: t('help.status.tunnel.title'),
      value: t('help.status.tunnel.running', { running, total }),
      detail: t('help.status.tunnel.connectionDetail', {
        count: data.overview.currentConnection,
        rtt: data.overview.averageRttMs,
      }),
      status: 'ok',
      icon: 'router',
    }
  }
  return {
    title: t('help.status.tunnel.title'),
    value: total ? t('help.states.allStopped') : t('help.states.notConfigured'),
    detail: total
      ? t('help.status.tunnel.configuredStopped', { count: total })
      : t('help.status.tunnel.emptyDetail'),
    status: 'warning',
    icon: 'router',
  }
})

function isRunningTunnelStatus(status: DashboardTunnel['status']): boolean {
  return ['running', 'starting', 'restarting', 'recovering'].includes(status)
}

const certificateStatus = computed<StatusItem>(() => {
  const list = certificates.value
  if (!list) {
    return {
      title: t('help.status.certificate.title'),
      value: errors.certificates ? t('help.states.unavailable') : t('help.states.waitingCheck'),
      detail: errors.certificates || t('help.status.certificate.waitingDetail'),
      status: errors.certificates ? 'error' : 'unknown',
      icon: 'shield-check',
    }
  }
  if (!list.certificates.length) {
    return {
      title: t('help.status.certificate.title'),
      value: t('help.states.noCertificate'),
      detail: t('help.status.certificate.emptyDetail'),
      status: 'warning',
      icon: 'shield-check',
    }
  }
  const severe = list.certificates.filter((cert) => isSevereCertificateStatus(cert.status))
  if (severe.length) {
    return {
      title: t('help.status.certificate.title'),
      value: t('help.status.certificate.abnormal', { count: severe.length }),
      detail: severe.map((cert) => cert.domain).join(t('help.punctuation.listSeparator')),
      status: 'error',
      icon: 'shield-check',
    }
  }
  const expiring = list.certificates.filter((cert) => cert.status === 'expiringSoon')
  if (expiring.length) {
    return {
      title: t('help.status.certificate.title'),
      value: t('help.status.certificate.expiring', { count: expiring.length }),
      detail: expiring.map((cert) => cert.domain).join(t('help.punctuation.listSeparator')),
      status: 'warning',
      icon: 'shield-check',
    }
  }
  return {
    title: t('help.status.certificate.title'),
    value: t('help.status.certificate.valid', { count: list.certificates.length }),
    detail: t('help.status.certificate.storeDetail', { path: list.storeRoot }),
    status: 'ok',
    icon: 'shield-check',
  }
})

const diagnosticChecklist = computed<ChecklistItem[]>(() => [
  serverConnectionCheck.value,
  networkCheck.value,
  portCheck.value,
  tokenCheck.value,
  versionCheck.value,
  timeSyncCheck.value,
  tlsCheck.value,
])

const serverConnectionCheck = computed<ChecklistItem>(() => {
  const report = connectionReport.value
  if (report) {
    return {
      id: 'server-connection',
      title: t('help.checks.serverConnection.title'),
      description: t('help.checks.serverConnection.description'),
      status: report.ok ? 'ok' : 'error',
      reason: connectionReportTitle(report),
      solution: connectionReportSolution(report),
      meta: `${report.elapsedMs}ms`,
    }
  }
  return {
    id: 'server-connection',
    title: t('help.checks.serverConnection.title'),
    description: t('help.checks.serverConnection.description'),
    status: 'warning',
    reason: selectedServer.value
      ? t('help.checks.serverConnection.pendingReason')
      : t('help.checks.serverConnection.noServerReason'),
    solution: selectedServer.value
      ? t('help.checks.serverConnection.pendingSolution')
      : t('help.checks.serverConnection.noServerSolution'),
    meta: selectedServerAddr.value ?? t('help.states.notConfigured'),
  }
})

const networkCheck = computed<ChecklistItem>(() => {
  const report = connectionReport.value
  if (!report) {
    return {
      id: 'network',
      title: t('help.checks.network.title'),
      description: t('help.checks.network.description'),
      status: 'warning',
      reason: t('help.checks.network.pendingReason'),
      solution: t('help.checks.network.pendingSolution'),
      meta: selectedServerAddr.value ?? t('help.states.notConfigured'),
    }
  }
  const ok = report.ok || report.code === 'TOKEN_ERROR'
  const networkFailed = [
    'DNS_ERROR',
    'TIMEOUT',
    'PORT_UNREACHABLE',
    'CLIENT_RUNTIME_UNAVAILABLE',
  ].includes(report.code)
  return {
    id: 'network',
    title: t('help.checks.network.title'),
    description: t('help.checks.network.description'),
    status: ok ? 'ok' : networkFailed ? 'error' : 'warning',
    reason: ok ? t('help.checks.network.okReason') : connectionReportReason(report),
    solution: ok ? t('help.checks.network.okSolution') : connectionReportSolution(report),
    meta: report.code,
  }
})

const portCheck = computed<ChecklistItem>(() => {
  const finding = deploymentReport.value?.findings.find((item) => item.id === 'server.port')
  if (!finding) {
    return {
      id: 'port',
      title: t('help.checks.port.title'),
      description: t('help.checks.port.description'),
      status: 'warning',
      reason: errors.deployment || t('help.checks.port.pendingReason'),
      solution: t('help.checks.port.pendingSolution'),
      meta: selectedServerAddr.value ?? t('help.states.notConfigured'),
    }
  }
  const localizedFinding = localizeDeploymentFinding(finding)
  return {
    id: 'port',
    title: t('help.checks.port.title'),
    description: t('help.checks.port.description'),
    status: normalizeDiagnosticStatus(localizedFinding.status),
    reason: localizedFinding.reason,
    solution: localizedFinding.solution,
    meta: finding.elapsedMs ? `${finding.elapsedMs}ms` : selectedServerAddr.value,
  }
})

const tokenCheck = computed<ChecklistItem>(() => {
  const report = connectionReport.value
  const server = selectedServer.value
  if (!server?.token) {
    return {
      id: 'token',
      title: t('help.checks.token.title'),
      description: t('help.checks.token.description'),
      status: 'warning',
      reason: t('help.checks.token.emptyReason'),
      solution: t('help.checks.token.emptySolution'),
    }
  }
  if (!report) {
    return {
      id: 'token',
      title: t('help.checks.token.title'),
      description: t('help.checks.token.description'),
      status: 'warning',
      reason: t('help.checks.token.pendingReason'),
      solution: t('help.checks.token.pendingSolution'),
    }
  }
  const tokenFailed = report.code === 'TOKEN_ERROR' || report.code === 'TOKEN_EMPTY'
  return {
    id: 'token',
    title: t('help.checks.token.title'),
    description: t('help.checks.token.description'),
    status: report.ok ? 'ok' : tokenFailed ? 'error' : 'warning',
    reason: report.ok ? t('help.checks.token.okReason') : connectionReportTitle(report),
    solution: report.ok ? t('help.checks.token.okSolution') : connectionReportSolution(report),
    meta: report.code,
  }
})

const versionCheck = computed<ChecklistItem>(() => {
  const info = systemInfo.value
  if (!info) {
    return {
      id: 'version',
      title: t('help.checks.version.title'),
      description: t('help.checks.version.description'),
      status: errors.system ? 'error' : 'warning',
      reason: errors.system || t('help.checks.version.pendingReason'),
      solution: t('help.checks.version.pendingSolution'),
    }
  }
  const serverUnknown = info.serverVersion === DIAGNOSTIC_VALUE_DISCONNECTED
  const mismatch = !serverUnknown && info.clientVersion !== info.serverVersion
  return {
    id: 'version',
    title: t('help.checks.version.title'),
    description: t('help.checks.version.description'),
    status: mismatch || serverUnknown ? 'warning' : 'ok',
    reason: serverUnknown
      ? t('help.checks.version.serverUnknownReason')
      : mismatch
        ? t('help.checks.version.mismatchReason', {
            client: info.clientVersion,
            server: info.serverVersion,
          })
        : t('help.checks.version.matchReason', { version: info.clientVersion }),
    solution: mismatch
      ? t('help.checks.version.mismatchSolution')
      : serverUnknown
        ? t('help.checks.version.serverUnknownSolution')
        : t('help.checks.version.matchSolution', { protocol: info.protocolVersion }),
    meta: info.protocolVersion,
  }
})

const timeSyncCheck = computed<ChecklistItem>(() => ({
  id: 'time-sync',
  title: t('help.checks.timeSync.title'),
  description: t('help.checks.timeSync.description'),
  status: health.value?.updatedAt || dashboard.value?.generatedAt ? 'warning' : 'warning',
  reason:
    health.value?.updatedAt || dashboard.value?.generatedAt
      ? t('help.checks.timeSync.localTimeReason', {
          time: formatTime(health.value?.updatedAt ?? dashboard.value?.generatedAt ?? 0),
        })
      : t('help.checks.timeSync.noTimestampReason'),
  solution: t('help.checks.timeSync.solution'),
  meta: lastGeneratedAt.value ? formatTime(lastGeneratedAt.value) : t('help.states.notGenerated'),
}))

const tlsCheck = computed<ChecklistItem>(() => {
  const tlsTunnels =
    dashboard.value?.tunnels.filter((tunnel) => tunnel.protocol === 'https' || tunnel.tls) ?? []
  const tlsErrors = tlsTunnels.reduce((sum, tunnel) => sum + (tunnel.tls?.errorCount ?? 0), 0)
  const certs = certificates.value?.certificates ?? []
  const severe = certs.filter((cert) => isSevereCertificateStatus(cert.status))
  if (tlsErrors || severe.length) {
    return {
      id: 'tls',
      title: t('help.checks.tls.title'),
      description: t('help.checks.tls.description'),
      status: 'error',
      reason: tlsErrors
        ? t('help.checks.tls.errorReason', { count: tlsErrors })
        : t('help.checks.tls.certificateErrorReason', { count: severe.length }),
      solution: t('help.checks.tls.errorSolution'),
      meta: `${tlsTunnels.length} HTTPS`,
    }
  }
  if (tlsTunnels.length || certs.length) {
    return {
      id: 'tls',
      title: t('help.checks.tls.title'),
      description: t('help.checks.tls.description'),
      status: 'ok',
      reason: t('help.checks.tls.okReason'),
      solution: t('help.checks.tls.okSolution'),
      meta: `${tlsTunnels.length} HTTPS`,
    }
  }
  return {
    id: 'tls',
    title: t('help.checks.tls.title'),
    description: t('help.checks.tls.description'),
    status: 'warning',
    reason: t('help.checks.tls.emptyReason'),
    solution: t('help.checks.tls.emptySolution'),
  }
})

const checklistSummary = computed(() => {
  const counts = diagnosticChecklist.value.reduce(
    (acc, item) => {
      acc[item.status] += 1
      return acc
    },
    { ok: 0, warning: 0, error: 0 },
  )
  return t('help.summary.checklist', counts)
})

const deploymentFindings = computed(() => deploymentReport.value?.findings ?? [])
const localizedDeploymentFindings = computed(() =>
  deploymentFindings.value.map((finding) => localizeDeploymentFinding(finding)),
)
const deploymentSummary = computed(() => {
  const report = deploymentReport.value
  if (!report) return ''
  return report.ok
    ? t('help.deployment.summaryOk')
    : t('help.deployment.summaryIssue', {
        count: report.findings.filter((finding) => finding.status !== 'ok').length,
      })
})

const reportFreshness = computed(() =>
  lastGeneratedAt.value
    ? t('help.report.updatedAt', { time: formatTime(lastGeneratedAt.value) })
    : t('help.states.notGenerated'),
)

const reportRows = computed(() => [
  {
    label: t('help.rows.systemInfo'),
    value: systemInfo.value ? t('help.states.collected') : t('help.states.unavailable'),
    muted: !systemInfo.value,
  },
  {
    label: t('help.rows.runtimeState'),
    value: dashboard.value ? t('help.states.collected') : t('help.states.unavailable'),
    muted: !dashboard.value,
  },
  {
    label: t('help.rows.tunnelState'),
    value: dashboard.value
      ? t('help.units.tunnels', { count: dashboard.value.tunnels.length })
      : t('help.states.unavailable'),
    muted: !dashboard.value,
  },
  {
    label: t('help.rows.connectionHistory'),
    value: t('help.units.records', { count: connectionHistory.value.length }),
    muted: !connectionHistory.value.length,
  },
  {
    label: t('help.rows.recentErrors'),
    value: t('help.units.entries', { count: recentErrors.value.length }),
    muted: !recentErrors.value.length,
  },
  {
    label: t('help.rows.logs'),
    value: t('help.units.entries', { count: logStore.logs.length }),
    muted: !logStore.logs.length,
  },
])

const systemRows = computed(() => {
  const info = systemInfo.value
  if (!info) {
    return [
      {
        label: t('help.rows.status'),
        value: errors.system || t('help.states.notCollected'),
        muted: true,
      },
      { label: t('help.rows.dataSource'), value: 'diagnostics_collect_system_info', muted: true },
    ]
  }
  return [
    { label: t('help.rows.clientVersion'), value: info.clientVersion },
    { label: t('help.rows.serverVersion'), value: localizeRuntimeValue(info.serverVersion) },
    { label: t('help.rows.protocolVersion'), value: info.protocolVersion },
    { label: 'Rust', value: info.rustVersion },
    { label: t('help.rows.system'), value: info.os },
    { label: t('help.rows.arch'), value: info.arch },
    { label: 'CPU', value: info.cpu },
    { label: t('help.rows.memory'), value: localizeRuntimeValue(info.memory) },
    { label: t('help.rows.configDir'), value: info.configDir },
    { label: t('help.rows.logDir'), value: info.logDir },
    { label: t('help.rows.currentDir'), value: info.currentDir },
  ]
})

const runtimeRows = computed(() => {
  const data = dashboard.value
  const active = selectedServer.value
  return [
    {
      label: t('help.rows.runtimeHealth'),
      value: health.value ? healthLabel(health.value.overall) : errors.health || t('help.states.notRead'),
      muted: !health.value,
    },
    {
      label: t('help.rows.server'),
      value: active
        ? `${active.name || active.host} (${active.status})`
        : errors.servers || t('help.states.notConfigured'),
      muted: !active,
    },
    {
      label: t('help.rows.tunnels'),
      value: data
        ? t('help.status.tunnel.running', {
            running: data.overview.runningTunnel,
            total: data.overview.tunnelCount,
          })
        : t('help.states.notRead'),
      muted: !data,
    },
    {
      label: t('help.rows.connections'),
      value: data
        ? t('help.units.currentConnections', { count: data.overview.currentConnection })
        : t('help.states.notRead'),
      muted: !data,
    },
    {
      label: t('help.rows.runtimeOnline'),
      value: data ? formatDuration(data.overview.runtimeUptimeSeconds) : t('help.states.notRead'),
      muted: !data,
    },
  ]
})

const recentErrors = computed<RecentErrorItem[]>(() => {
  const failedConnections = connectionHistory.value
    .filter((entry) => entry.result === 'failed')
    .slice(0, 4)
    .map((entry) => ({
      id: `history-${entry.id}`,
      source: 'Connection',
      title: entry.serverAddr,
      detail: entry.failureReason || t('help.states.connectionFailed'),
    }))

  const logErrors = logStore.logs
    .filter((log) => log.level === 'ERROR' || log.level === 'FATAL')
    .slice(-6)
    .reverse()
    .map((log) => ({
      id: `log-${log.id}`,
      source: log.source,
      title: log.level,
      detail: log.message,
    }))

  return [...failedConnections, ...logErrors].slice(0, 8)
})

onMounted(() => {
  void refreshDiagnostics({ silent: true })
})

async function refreshDiagnostics(options: { silent?: boolean } = {}) {
  const startedAt = Date.now()
  loading.value = true
  diagnosticRunTone.value = 'info'
  setDiagnosticRunMessage('info', 'help.run.running')
  resetErrors()
  try {
    // 统一从现有 IPC、服务和日志 Store 聚合数据，不在前端制造状态。
    const [nextSystem, nextServers, nextDashboard, nextHealth, nextCertificates] =
      await Promise.all([
        capture('system', () => diagnosticsService.collectSystemInfo()),
        capture('servers', () => serverService.list()),
        capture('runtime', () => ipc.invoke<DashboardData>('runtime_get_dashboard')),
        capture('health', () => ipc.invoke<HealthReport>('runtime_get_health')),
        capture('certificates', () => certificateService.list()),
      ])

    systemInfo.value = nextSystem
    serverList.value = nextServers
    dashboard.value = nextDashboard
    health.value = nextHealth
    certificates.value = nextCertificates

    const server = selectedServer.value
    const [nextDeployment, nextConnection] = await Promise.all([
      capture('deployment', () => diagnosticsService.runDeployment(selectedServerAddr.value)),
      server
        ? capture('connection', () => diagnosticsService.testConnection(toConnectionInput(server)))
        : Promise.resolve(null),
      capture('logs', loadLogs),
    ])

    deploymentReport.value = nextDeployment
    connectionReport.value = nextConnection
    refreshMemory()
    lastGeneratedAt.value = Date.now()

    const summary = buildRunSummary()
    diagnosticRunTone.value = summary.tone
    diagnosticRunMessageKey.value = summary
    if (!options.silent) {
      toast[summary.tone](diagnosticRunMessage.value)
    }
  } catch (error) {
    const message = toErrorMessage(error)
    diagnosticRunTone.value = 'error'
    diagnosticRunMessageKey.value = {
      key: 'help.run.failed',
      params: { message },
    }
    if (!options.silent) toast.error(diagnosticRunMessage.value)
  } finally {
    await waitForVisibleLoading(startedAt)
    loading.value = false
  }
}

async function loadLogs() {
  await logStore.refresh()
  if (logStore.status === 'error') {
    throw new Error(logStore.error || t('help.errors.logLoadFailed'))
  }
}

function buildRunSummary(): DiagnosticRunSummary {
  const counts = diagnosticChecklist.value.reduce(
    (acc, item) => {
      acc[item.status] += 1
      return acc
    },
    { ok: 0, warning: 0, error: 0 },
  )
  const sourceErrors = Object.values(errors).filter(Boolean).length

  if (counts.error || sourceErrors) {
    return {
      tone: 'error' as const,
      key: 'help.run.completedWithErrors',
      params: { errors: counts.error, warnings: counts.warning },
    }
  }

  if (!selectedServer.value) {
    return {
      tone: 'warning' as const,
      key: 'help.run.completedNoServer',
    }
  }

  if (counts.warning) {
    return {
      tone: 'warning' as const,
      key: 'help.run.completedWithWarnings',
      params: { ok: counts.ok, warnings: counts.warning },
    }
  }

  return {
    tone: 'success' as const,
    key: 'help.run.completedSuccess',
    params: { ok: counts.ok },
  }
}

async function capture<T>(scope: DiagnosticScope, task: () => Promise<T>): Promise<T | null> {
  try {
    return await task()
  } catch (error) {
    errors[scope] = toErrorMessage(error)
    return null
  }
}

function refreshMemory() {
  recentServers.value = diagnosticsService.getRecentServers()
  connectionHistory.value = diagnosticsService.getConnectionHistory()
}

function setDiagnosticRunMessage(
  tone: 'info' | 'success' | 'warning' | 'error',
  key: string,
  params?: Record<string, unknown>,
) {
  diagnosticRunTone.value = tone
  diagnosticRunMessageKey.value = { key, params }
}

async function copyDiagnosticReport(showToast = true) {
  if (!lastGeneratedAt.value && !loading.value) {
    await refreshDiagnostics()
  }
  await diagnosticsService.copyText(JSON.stringify(buildReportPayload(), null, 2))
  if (showToast) toast.success(t('help.toast.reportCopied'))
}

async function copySystemInfo() {
  if (!systemInfo.value) {
    toast.warning(t('help.toast.systemInfoUnavailable'))
    return
  }
  await diagnosticsService.copyText(JSON.stringify(systemInfo.value, null, 2))
  toast.success(t('help.toast.systemInfoCopied'))
}

async function exportDebugBundle() {
  if (!lastGeneratedAt.value && !loading.value) {
    await refreshDiagnostics()
  }
  downloadJson(`gate-debug-bundle-${timestampForFilename()}.json`, buildReportPayload())
  toast.success(t('help.toast.bundleExported'))
}

async function openIssue() {
  await copyDiagnosticReport(false)
  await openUrl(GITHUB_ISSUE_URL)
  toast.success(t('help.toast.issueReady'))
}

async function openLogDirectory() {
  await openDirectory(systemInfo.value?.logDir, t('help.rows.logDir'))
}

async function openConfigDirectory() {
  await openDirectory(systemInfo.value?.configDir, t('help.rows.configDir'))
}

function clearSupportCache() {
  diagnosticsService.clearSupportCache()
  refreshMemory()
  toast.success(t('help.toast.cacheCleared'))
}

async function exportLogs() {
  if (!logStore.logs.length) {
    await capture('logs', loadLogs)
  }
  downloadLogs(logStore.logs, 'txt')
  toast.success(t('help.toast.logsExported'))
}

function buildReportPayload() {
  // 报告只打包用户当前可见的诊断上下文，避免触碰 Runtime 或 Backend。
  return {
    generatedAt: new Date().toISOString(),
    sources: {
      systemInfo: Boolean(systemInfo.value),
      runtimeState: Boolean(dashboard.value),
      tunnelState: Boolean(dashboard.value?.tunnels),
      certificateState: Boolean(certificates.value),
      connectionHistory: connectionHistory.value.length,
      logs: logStore.logs.length,
    },
    errors: { ...errors },
    systemInfo: systemInfo.value,
    runtimeState: {
      dashboard: dashboard.value,
      health: health.value,
      servers: serverList.value,
    },
    tunnelState: dashboard.value?.tunnels ?? [],
    certificateState: certificates.value,
    diagnostics: {
      connection: connectionReport.value,
      deployment: deploymentReport.value,
      checklist: diagnosticChecklist.value,
    },
    connectionHistory: connectionHistory.value,
    recentServers: recentServers.value,
    recentErrors: recentErrors.value,
    logs: logStore.logs.slice(-200),
  }
}

function toConnectionInput(server: RuntimeServerRecord) {
  return {
    host: server.host,
    port: server.port,
    token: server.token,
  }
}

function resetErrors() {
  for (const key of Object.keys(errors) as DiagnosticScope[]) {
    errors[key] = ''
  }
}

function normalizeDiagnosticStatus(status: DiagnosticStatus): ChecklistStatus {
  if (status === 'ok') return 'ok'
  if (status === 'warning') return 'warning'
  return 'error'
}

function connectionReportTitle(report: ConnectionTestReport) {
  return connectionReportText(report, 'title')
}

function connectionReportReason(report: ConnectionTestReport) {
  return connectionReportText(report, 'reason')
}

function connectionReportSolution(report: ConnectionTestReport) {
  return connectionReportText(report, 'solution')
}

function connectionReportText(
  report: ConnectionTestReport,
  field: 'title' | 'reason' | 'solution',
) {
  const key = `help.connectionReport.${toLocaleCode(report.code)}.${field}`
  if (te(key)) {
    return t(key, {
      serverAddr: selectedServerAddr.value ?? '',
      reason: report.reason,
      title: report.title,
      solution: report.solution,
    })
  }
  return report[field]
}

function localizeDeploymentFinding(finding: DiagnosticFinding): DiagnosticFinding {
  const baseKey = `help.deployment.findings.${toLocaleCode(finding.id)}`
  const params = {
    reason: finding.reason,
    serverAddr: selectedServerAddr.value ?? '',
  }
  return {
    ...finding,
    label: localizeDiagnosticText(finding.label, `${baseKey}.label`, params),
    reason: localizeDiagnosticText(finding.reason, `${baseKey}.reason`, params),
    possibleCause: localizeDiagnosticText(
      finding.possibleCause,
      `${baseKey}.possibleCause`,
      params,
    ),
    solution: localizeDiagnosticText(finding.solution, `${baseKey}.solution`, params),
  }
}

function localizeDiagnosticText(
  value: string,
  fallbackKey: string,
  params: Record<string, unknown>,
) {
  if (te(value)) return t(value, params)
  if (te(fallbackKey)) return t(fallbackKey, { ...params, reason: value })
  return value
}

function toLocaleCode(value: string) {
  return value
    .toLowerCase()
    .replace(/[^a-z0-9]+([a-z0-9])/g, (_, char: string) => char.toUpperCase())
}

function findingIcon(status: DiagnosticStatus) {
  if (status === 'ok') return 'check-circle'
  if (status === 'warning') return 'alert-triangle'
  return 'alert-circle'
}

function mapHealthStatus(status: HealthStatus): SupportStatus {
  if (status === 'healthy') return 'ok'
  if (status === 'warning') return 'warning'
  return 'error'
}

function healthLabel(status: HealthStatus) {
  if (status === 'healthy') return t('help.health.healthy')
  if (status === 'warning') return t('help.health.warning')
  if (status === 'critical') return t('help.health.critical')
  return t('help.health.offline')
}

function isSevereCertificateStatus(status: CertificateStatus) {
  return status === 'expired' || status === 'revoked' || status === 'deleted' || status === 'failed'
}

async function openDirectory(path: string | undefined, label: string) {
  if (!path) {
    toast.warning(t('help.toast.directoryUnavailable', { label }))
    return
  }
  try {
    await openExternal(path)
  } catch {
    await diagnosticsService.copyText(path)
    toast.warning(t('help.toast.directoryCopied', { label }))
  }
}

async function openUrl(url: string) {
  try {
    await openExternal(url)
  } catch {
    window.open(url, '_blank', 'noopener,noreferrer')
  }
}

function downloadJson(filename: string, payload: unknown) {
  const blob = new Blob([JSON.stringify(payload, null, 2)], {
    type: 'application/json;charset=utf-8',
  })
  const url = URL.createObjectURL(blob)
  const link = document.createElement('a')
  link.href = url
  link.download = filename
  link.click()
  URL.revokeObjectURL(url)
}

function timestampForFilename() {
  return new Date().toISOString().replace(/[:.]/g, '-')
}

function formatTime(timestamp: number) {
  if (!timestamp) return t('help.states.unknown')
  return new Date(timestamp).toLocaleString()
}

function localizeRuntimeValue(value: string) {
  if (value === DIAGNOSTIC_VALUE_DISCONNECTED) return t('help.states.disconnected')
  if (value === DIAGNOSTIC_VALUE_MEMORY_PERMISSION_REQUIRED)
    return t('help.system.memoryNeedsPermission')
  if (value === 'unknown') return t('help.states.unknown')
  return value
}

function formatDuration(seconds: number) {
  if (seconds < 60) return `${seconds}s`
  const minutes = Math.floor(seconds / 60)
  if (minutes < 60) return `${minutes}m`
  const hours = Math.floor(minutes / 60)
  return `${hours}h ${minutes % 60}m`
}

function toErrorMessage(error: unknown) {
  return error instanceof Error ? error.message : String(error)
}

async function waitForVisibleLoading(startedAt: number) {
  const elapsed = Date.now() - startedAt
  if (elapsed >= 350) return
  await new Promise((resolve) => window.setTimeout(resolve, 350 - elapsed))
}
</script>

<style scoped>
.help-center-page {
  width: min(100%, var(--content-max-width));
  height: 100%;
  min-height: 0;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: var(--space-5);
  color: var(--text-primary);
}

.help-hero {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  gap: var(--space-4);
  flex-shrink: 0;
}

.help-hero p,
.section-heading p {
  color: var(--text-tertiary);
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
  text-transform: uppercase;
}

.help-hero h1 {
  margin-top: 2px;
  font-size: var(--text-3xl);
  font-weight: var(--weight-semibold);
  letter-spacing: 0;
}

.help-hero span {
  display: block;
  margin-top: var(--space-1);
  color: var(--text-secondary);
  line-height: var(--leading-normal);
}

.help-hero__actions {
  display: flex;
  flex-wrap: wrap;
  justify-content: flex-end;
  align-items: center;
  gap: var(--space-2);
}

.diagnostic-run-state {
  min-height: var(--control-height-md);
  display: inline-flex;
  align-items: center;
  gap: var(--space-2);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
  color: var(--text-secondary);
  padding: 0 var(--space-3);
  font-size: var(--text-sm);
  white-space: nowrap;
}

.diagnostic-run-state.success {
  border-color: rgba(47, 209, 124, 0.28);
  background: var(--color-success-muted);
  color: var(--color-success);
}

.diagnostic-run-state.warning {
  border-color: rgba(245, 158, 11, 0.28);
  background: var(--color-warning-muted);
  color: var(--color-warning);
}

.diagnostic-run-state.error {
  border-color: rgba(255, 92, 92, 0.28);
  background: var(--color-error-muted);
  color: var(--color-error);
}

.status-grid {
  display: grid;
  grid-template-columns: repeat(5, minmax(0, 1fr));
  gap: var(--space-3);
}

.help-layout {
  min-height: 0;
  display: grid;
  grid-template-columns: minmax(0, 1fr) 360px;
  gap: var(--space-5);
}

.help-main,
.help-side {
  min-height: 0;
  overflow: auto;
  display: flex;
  flex-direction: column;
  gap: var(--space-5);
  padding-bottom: var(--space-3);
}

.help-section,
.advanced-tools {
  display: grid;
  gap: var(--space-4);
}

.section-heading {
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: var(--space-3);
}

.section-heading.compact {
  align-items: flex-start;
}

.section-heading h2 {
  margin-top: 2px;
  color: var(--text-primary);
  font-size: var(--text-xl);
  font-weight: var(--weight-semibold);
}

.section-heading > span {
  color: var(--text-secondary);
  font-size: var(--text-sm);
  text-align: right;
  overflow-wrap: anywhere;
}

.diagnostic-list {
  display: grid;
  gap: var(--space-3);
}

.finding-strip {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  gap: var(--space-3);
}

.finding-strip article {
  min-height: 76px;
  display: grid;
  grid-template-columns: 22px minmax(0, 1fr);
  gap: var(--space-3);
  padding: var(--space-3);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
}

.finding-strip article.is-ok svg {
  color: var(--color-success);
}

.finding-strip article.is-warning svg {
  color: var(--color-warning);
}

.finding-strip article.is-error svg {
  color: var(--color-error);
}

.finding-strip strong {
  color: var(--text-primary);
}

.finding-strip p {
  margin-top: 2px;
  color: var(--text-secondary);
  line-height: var(--leading-normal);
  overflow-wrap: anywhere;
}

.report-grid,
.tool-grid {
  display: grid;
  grid-template-columns: repeat(3, minmax(0, 1fr));
  gap: var(--space-3);
}

.tool-grid {
  grid-template-columns: 1fr;
}

.recent-errors {
  display: grid;
  gap: var(--space-2);
}

.recent-errors header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  color: var(--text-secondary);
}

.recent-errors header strong {
  color: var(--text-primary);
  font-size: var(--text-lg);
}

.recent-errors header span {
  color: var(--text-tertiary);
  font-size: var(--text-sm);
}

.recent-errors article {
  min-height: 56px;
  display: grid;
  grid-template-columns: 92px minmax(0, 1fr);
  gap: var(--space-3);
  padding: var(--space-3);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-md);
  background: var(--bg-surface);
}

.recent-errors article > span {
  color: var(--color-error);
  font-size: var(--text-xs);
  font-weight: var(--weight-semibold);
  text-transform: uppercase;
}

.recent-errors article strong {
  color: var(--text-primary);
}

.recent-errors article p,
.empty-copy {
  margin-top: 2px;
  color: var(--text-secondary);
  line-height: var(--leading-normal);
  overflow-wrap: anywhere;
}

@media (max-width: 1280px) {
  .status-grid {
    grid-template-columns: repeat(3, minmax(0, 1fr));
  }

  .help-layout {
    grid-template-columns: 1fr;
  }

  .help-side {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .advanced-tools {
    grid-column: 1 / -1;
  }
}

@media (max-width: 860px) {
  .help-hero,
  .section-heading {
    flex-direction: column;
    align-items: flex-start;
  }

  .help-hero__actions {
    justify-content: flex-start;
  }

  .status-grid,
  .finding-strip,
  .report-grid,
  .help-side {
    grid-template-columns: 1fr;
  }

  .recent-errors article {
    grid-template-columns: 1fr;
  }
}
</style>
