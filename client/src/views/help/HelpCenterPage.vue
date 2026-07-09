<template>
  <section class="help-center-page">
    <header class="help-hero">
      <div>
        <p>Support Center</p>
        <h1>帮助与诊断</h1>
        <span>检测 Gate 状态，生成诊断报告，帮助定位连接和部署问题。</span>
      </div>
      <div class="help-hero__actions">
        <div v-if="diagnosticRunMessage" class="diagnostic-run-state" :class="diagnosticRunTone">
          <GIcon :name="loading ? 'loader' : diagnosticRunIcon" :size="14" :spin="loading" />
          <span>{{ diagnosticRunMessage }}</span>
        </div>
        <GButton variant="primary" icon="activity" :loading="loading" @click="refreshDiagnostics()">
          {{ loading ? '诊断中' : '开始诊断' }}
        </GButton>
        <GButton variant="secondary" icon="copy" @click="copyDiagnosticReport()">
          复制诊断报告
        </GButton>
      </div>
    </header>

    <section class="help-section" aria-label="System Status">
      <header class="section-heading">
        <div>
          <p>System Status</p>
          <h2>系统状态</h2>
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
              <p>Diagnostic Checklist</p>
              <h2>诊断清单</h2>
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
              <p>Deployment Findings</p>
              <h2>部署检查结果</h2>
            </div>
            <span>{{ deploymentReport?.summary }}</span>
          </header>

          <div class="finding-strip">
            <article
              v-for="finding in deploymentFindings"
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
              <p>Report Center</p>
              <h2>报告中心</h2>
            </div>
            <span>{{ reportFreshness }}</span>
          </header>

          <div class="report-grid">
            <ReportCard
              title="复制诊断信息"
              description="复制可直接粘贴到 Issue 或协作工具中的结构化报告。"
              icon="copy"
              tone="primary"
              @action="copyDiagnosticReport" />
            <ReportCard
              title="导出 Debug Bundle"
              description="导出包含系统、运行时、隧道、错误和日志摘要的 JSON 文件。"
              icon="package"
              @action="exportDebugBundle" />
            <ReportCard
              title="打开 GitHub Issue"
              description="打开问题提交入口，并先复制当前诊断报告。"
              icon="github"
              @action="openIssue" />
          </div>

          <SystemInfoCard
            title="报告内容"
            description="报告只聚合已有前端可见能力。"
            :rows="reportRows" />

          <div class="recent-errors">
            <header>
              <strong>Recent Errors</strong>
              <span>{{ recentErrors.length }} 条</span>
            </header>
            <article v-for="error in recentErrors" :key="error.id">
              <span>{{ error.source }}</span>
              <div>
                <strong>{{ error.title }}</strong>
                <p>{{ error.detail }}</p>
              </div>
            </article>
            <p v-if="!recentErrors.length" class="empty-copy">暂无近期错误。</p>
          </div>
        </section>
      </main>

      <aside class="help-side">
        <SystemInfoCard
          title="System Info"
          description="来自桌面运行时的系统信息。"
          :rows="systemRows">
          <template #actions>
            <GButton variant="ghost" size="sm" icon="copy" @click="copySystemInfo" />
          </template>
        </SystemInfoCard>

        <SystemInfoCard
          title="Runtime State"
          description="运行时、隧道和服务器快照。"
          :rows="runtimeRows" />

        <section class="advanced-tools">
          <header class="section-heading compact">
            <div>
              <p>Advanced Tools</p>
              <h2>高级工具</h2>
            </div>
          </header>

          <div class="tool-grid">
            <ReportCard
              title="打开日志目录"
              :description="systemInfo?.logDir ?? '等待系统信息'"
              icon="logs"
              @action="openLogDirectory" />
            <ReportCard
              title="打开配置目录"
              :description="systemInfo?.configDir ?? '等待系统信息'"
              icon="settings"
              @action="openConfigDirectory" />
            <ReportCard
              title="刷新诊断"
              description="重新读取状态、日志、证书和部署检查。"
              icon="refresh"
              @action="refreshDiagnostics" />
            <ReportCard
              title="清理缓存"
              description="清理帮助中心使用的最近服务器和连接历史。"
              icon="trash"
              tone="danger"
              @action="clearSupportCache" />
            <ReportCard
              title="导出日志"
              description="导出当前可见运行时日志文本。"
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
import { open as openExternal } from '@tauri-apps/plugin-shell'
import GButton from '@components/base/GButton.vue'
import GIcon from '@components/icons/GIcon.vue'
import { useFeedback } from '@composables/useFeedback'
import { TauriIpcClient } from '@/ipc'
import { GITHUB_ISSUE_URL } from '@/constants'
import { diagnosticsService, serverService } from '@/services'
import type {
  ConnectionHistoryEntry,
  ConnectionTestReport,
  DeploymentCheckReport,
  DiagnosticStatus,
  RecentServer,
  RuntimeServerList,
  RuntimeServerRecord,
  SystemInfoReport,
} from '@/services'
import type { CertificateListResponse, CertificateStatus } from '@views/certificates/types'
import { certificateService } from '@views/certificates/service'
import type { DashboardData, HealthReport, HealthStatus } from '@/monitoring/types'
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

const ipc = new TauriIpcClient()
const logStore = useLogStore()
const { toast } = useFeedback()

const loading = ref(false)
const lastGeneratedAt = ref(0)
const diagnosticRunMessage = ref('')
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

const clientStatus = computed<StatusItem>(() => {
  if (systemInfo.value) {
    return {
      title: 'Client Status',
      value: `v${systemInfo.value.clientVersion}`,
      detail: `${systemInfo.value.os} / ${systemInfo.value.arch}`,
      status: 'ok',
      icon: 'monitor',
    }
  }
  return {
    title: 'Client Status',
    value: errors.system ? '不可用' : '等待检测',
    detail: errors.system || '尚未读取桌面客户端信息。',
    status: errors.system ? 'error' : 'unknown',
    icon: 'monitor',
  }
})

const serverStatus = computed<StatusItem>(() => {
  const list = serverList.value
  if (!list) {
    return {
      title: 'Server Status',
      value: errors.servers ? '不可用' : '等待检测',
      detail: errors.servers || '尚未读取服务器列表。',
      status: errors.servers ? 'error' : 'unknown',
      icon: 'servers',
    }
  }
  const active = selectedServer.value
  if (list.connected && active) {
    return {
      title: 'Server Status',
      value: '已连接',
      detail: `${active.name || active.host} · ${active.host}:${active.port}`,
      status: 'ok',
      icon: 'servers',
    }
  }
  if (list.items.length) {
    return {
      title: 'Server Status',
      value: '未连接',
      detail: `${list.items.length} 个服务器配置可用，当前没有活跃连接。`,
      status: 'warning',
      icon: 'servers',
    }
  }
  return {
    title: 'Server Status',
    value: '未配置',
    detail: '尚未添加 Gate Server。',
    status: 'warning',
    icon: 'servers',
  }
})

const runtimeStatus = computed<StatusItem>(() => {
  const runtimeError = errors.runtime || errors.health
  if (runtimeError) {
    return {
      title: 'Runtime Status',
      value: '不可用',
      detail: runtimeError,
      status: 'error',
      icon: 'cpu',
    }
  }
  if (!health.value) {
    return {
      title: 'Runtime Status',
      value: '等待检测',
      detail: '尚未读取运行时健康状态。',
      status: 'unknown',
      icon: 'cpu',
    }
  }
  return {
    title: 'Runtime Status',
    value: healthLabel(health.value.overall),
    detail: `${health.value.signals.length} 个健康信号 · ${formatTime(health.value.updatedAt)}`,
    status: mapHealthStatus(health.value.overall),
    icon: 'cpu',
  }
})

const tunnelStatus = computed<StatusItem>(() => {
  const data = dashboard.value
  if (!data) {
    return {
      title: 'Tunnel Status',
      value: errors.runtime ? '不可用' : '等待检测',
      detail: errors.runtime || '尚未读取隧道状态。',
      status: errors.runtime ? 'error' : 'unknown',
      icon: 'router',
    }
  }
  const total = data.tunnels.length
  const running = data.tunnels.filter((tunnel) => tunnel.status === 'running').length
  if (running > 0) {
    return {
      title: 'Tunnel Status',
      value: `${running}/${total} 运行中`,
      detail: `当前连接 ${data.overview.currentConnection} 个，平均 RTT ${data.overview.averageRttMs}ms。`,
      status: 'ok',
      icon: 'router',
    }
  }
  return {
    title: 'Tunnel Status',
    value: total ? '全部停止' : '未配置',
    detail: total ? `${total} 个隧道已配置但未运行。` : '暂无隧道数据。',
    status: 'warning',
    icon: 'router',
  }
})

const certificateStatus = computed<StatusItem>(() => {
  const list = certificates.value
  if (!list) {
    return {
      title: 'Certificate Status',
      value: errors.certificates ? '不可用' : '等待检测',
      detail: errors.certificates || '尚未读取证书状态。',
      status: errors.certificates ? 'error' : 'unknown',
      icon: 'shield-check',
    }
  }
  if (!list.certificates.length) {
    return {
      title: 'Certificate Status',
      value: '无证书',
      detail: '证书库为空，HTTPS 隧道会依赖后续配置。',
      status: 'warning',
      icon: 'shield-check',
    }
  }
  const severe = list.certificates.filter((cert) => isSevereCertificateStatus(cert.status))
  if (severe.length) {
    return {
      title: 'Certificate Status',
      value: `${severe.length} 个异常`,
      detail: severe.map((cert) => cert.domain).join('、'),
      status: 'error',
      icon: 'shield-check',
    }
  }
  const expiring = list.certificates.filter((cert) => cert.status === 'expiringSoon')
  if (expiring.length) {
    return {
      title: 'Certificate Status',
      value: `${expiring.length} 个即将过期`,
      detail: expiring.map((cert) => cert.domain).join('、'),
      status: 'warning',
      icon: 'shield-check',
    }
  }
  return {
    title: 'Certificate Status',
    value: `${list.certificates.length} 个有效`,
    detail: `证书库：${list.storeRoot}`,
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
      title: 'Server Connection Check',
      description: '检查客户端能否连接到当前 Gate Server。',
      status: report.ok ? 'ok' : 'error',
      reason: report.title,
      solution: report.solution,
      meta: `${report.elapsedMs}ms`,
    }
  }
  return {
    id: 'server-connection',
    title: 'Server Connection Check',
    description: '检查客户端能否连接到当前 Gate Server。',
    status: 'warning',
    reason: selectedServer.value ? '尚未运行连接测试。' : '没有可用于测试的服务器配置。',
    solution: selectedServer.value
      ? '点击开始诊断重新执行连接测试。'
      : '先在服务器页面添加 Gate Server。',
    meta: selectedServerAddr.value ?? '未配置',
  }
})

const networkCheck = computed<ChecklistItem>(() => {
  const report = connectionReport.value
  if (!report) {
    return {
      id: 'network',
      title: 'Network Check',
      description: '检查 DNS、网络链路和连接超时。',
      status: 'warning',
      reason: '等待连接诊断结果。',
      solution: '开始诊断后会基于真实连接结果判断网络状态。',
      meta: selectedServerAddr.value ?? '未配置',
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
    title: 'Network Check',
    description: '检查 DNS、网络链路和连接超时。',
    status: ok ? 'ok' : networkFailed ? 'error' : 'warning',
    reason: ok ? '网络链路已到达服务端。' : report.reason,
    solution: ok ? '继续检查认证和协议版本。' : report.solution,
    meta: report.code,
  }
})

const portCheck = computed<ChecklistItem>(() => {
  const finding = deploymentReport.value?.findings.find((item) => item.id === 'server.port')
  if (!finding) {
    return {
      id: 'port',
      title: 'Port Check',
      description: '检查服务端端口是否可解析、可访问。',
      status: 'warning',
      reason: errors.deployment || '尚未收到端口检查结果。',
      solution: '开始诊断后会读取桌面运行时的部署检查。',
      meta: selectedServerAddr.value ?? '未配置',
    }
  }
  return {
    id: 'port',
    title: 'Port Check',
    description: '检查服务端端口是否可解析、可访问。',
    status: normalizeDiagnosticStatus(finding.status),
    reason: finding.reason,
    solution: finding.solution,
    meta: finding.elapsedMs ? `${finding.elapsedMs}ms` : selectedServerAddr.value,
  }
})

const tokenCheck = computed<ChecklistItem>(() => {
  const report = connectionReport.value
  const server = selectedServer.value
  if (!server?.token) {
    return {
      id: 'token',
      title: 'Token Check',
      description: '检查当前服务器 Token 是否可通过认证。',
      status: 'warning',
      reason: '当前服务器没有可用于测试的 Token。',
      solution: '在服务器配置中填入 Token 后重新诊断。',
    }
  }
  if (!report) {
    return {
      id: 'token',
      title: 'Token Check',
      description: '检查当前服务器 Token 是否可通过认证。',
      status: 'warning',
      reason: '等待连接测试返回认证结果。',
      solution: '点击开始诊断。',
    }
  }
  const tokenFailed = report.code === 'TOKEN_ERROR' || report.code === 'TOKEN_EMPTY'
  return {
    id: 'token',
    title: 'Token Check',
    description: '检查当前服务器 Token 是否可通过认证。',
    status: report.ok ? 'ok' : tokenFailed ? 'error' : 'warning',
    reason: report.ok ? 'Token 已通过服务端认证。' : report.title,
    solution: report.ok ? '继续创建或恢复隧道。' : report.solution,
    meta: report.code,
  }
})

const versionCheck = computed<ChecklistItem>(() => {
  const info = systemInfo.value
  if (!info) {
    return {
      id: 'version',
      title: 'Version Check',
      description: '检查客户端、服务端和协议版本。',
      status: errors.system ? 'error' : 'warning',
      reason: errors.system || '尚未读取版本信息。',
      solution: '在 Gate Desktop 中重新打开帮助中心。',
    }
  }
  const serverUnknown = info.serverVersion === '未连接'
  const mismatch = !serverUnknown && info.clientVersion !== info.serverVersion
  return {
    id: 'version',
    title: 'Version Check',
    description: '检查客户端、服务端和协议版本。',
    status: mismatch || serverUnknown ? 'warning' : 'ok',
    reason: serverUnknown
      ? '服务端版本暂未返回。'
      : mismatch
        ? `客户端 ${info.clientVersion}，服务端 ${info.serverVersion}。`
        : `客户端与服务端版本一致：${info.clientVersion}。`,
    solution: mismatch
      ? '建议升级到同一版本后重试。'
      : serverUnknown
        ? '连接服务器后重新诊断。'
        : `协议版本 ${info.protocolVersion} 可用。`,
    meta: info.protocolVersion,
  }
})

const timeSyncCheck = computed<ChecklistItem>(() => ({
  id: 'time-sync',
  title: 'Time Sync',
  description: '检查诊断数据时间戳是否可用于定位问题。',
  status: health.value?.updatedAt || dashboard.value?.generatedAt ? 'warning' : 'warning',
  reason:
    health.value?.updatedAt || dashboard.value?.generatedAt
      ? `本地运行时最近更新时间：${formatTime(health.value?.updatedAt ?? dashboard.value?.generatedAt ?? 0)}。`
      : '当前诊断命令没有返回服务端时间戳。',
  solution: '如遇认证过期或证书异常，请同时确认本机和服务器系统时间。',
  meta: lastGeneratedAt.value ? formatTime(lastGeneratedAt.value) : '未生成',
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
      title: 'TLS Check',
      description: '检查 HTTPS 隧道和证书状态。',
      status: 'error',
      reason: tlsErrors ? `TLS 错误计数 ${tlsErrors}。` : `${severe.length} 个证书异常。`,
      solution: '查看证书页和日志，修复证书状态后重新诊断。',
      meta: `${tlsTunnels.length} HTTPS`,
    }
  }
  if (tlsTunnels.length || certs.length) {
    return {
      id: 'tls',
      title: 'TLS Check',
      description: '检查 HTTPS 隧道和证书状态。',
      status: 'ok',
      reason: '未发现 TLS 错误或异常证书。',
      solution: '继续保持证书自动续期和日志观察。',
      meta: `${tlsTunnels.length} HTTPS`,
    }
  }
  return {
    id: 'tls',
    title: 'TLS Check',
    description: '检查 HTTPS 隧道和证书状态。',
    status: 'warning',
    reason: '当前没有 HTTPS 隧道或证书数据。',
    solution: '如果要排查 HTTPS，请先创建证书或 HTTPS 隧道后重新诊断。',
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
  return `${counts.ok} 通过 · ${counts.warning} 警告 · ${counts.error} 失败`
})

const deploymentFindings = computed(() => deploymentReport.value?.findings ?? [])

const reportFreshness = computed(() =>
  lastGeneratedAt.value ? `更新于 ${formatTime(lastGeneratedAt.value)}` : '尚未生成',
)

const reportRows = computed(() => [
  { label: 'System Info', value: systemInfo.value ? '已采集' : '不可用', muted: !systemInfo.value },
  { label: 'Runtime State', value: dashboard.value ? '已采集' : '不可用', muted: !dashboard.value },
  {
    label: 'Tunnel State',
    value: dashboard.value ? `${dashboard.value.tunnels.length} 个隧道` : '不可用',
    muted: !dashboard.value,
  },
  {
    label: 'Connection History',
    value: `${connectionHistory.value.length} 条记录`,
    muted: !connectionHistory.value.length,
  },
  {
    label: 'Recent Errors',
    value: `${recentErrors.value.length} 条`,
    muted: !recentErrors.value.length,
  },
  { label: 'Logs', value: `${logStore.logs.length} 条`, muted: !logStore.logs.length },
])

const systemRows = computed(() => {
  const info = systemInfo.value
  if (!info) {
    return [
      { label: '状态', value: errors.system || '尚未采集', muted: true },
      { label: '数据源', value: 'diagnostics_collect_system_info', muted: true },
    ]
  }
  return [
    { label: '客户端版本', value: info.clientVersion },
    { label: '服务端版本', value: info.serverVersion },
    { label: '协议版本', value: info.protocolVersion },
    { label: 'Rust', value: info.rustVersion },
    { label: '系统', value: info.os },
    { label: '架构', value: info.arch },
    { label: 'CPU', value: info.cpu },
    { label: '内存', value: info.memory },
    { label: '配置目录', value: info.configDir },
    { label: '日志目录', value: info.logDir },
    { label: '工作目录', value: info.currentDir },
  ]
})

const runtimeRows = computed(() => {
  const data = dashboard.value
  const active = selectedServer.value
  return [
    {
      label: '运行时健康',
      value: health.value ? healthLabel(health.value.overall) : errors.health || '未读取',
      muted: !health.value,
    },
    {
      label: '服务器',
      value: active
        ? `${active.name || active.host} (${active.status})`
        : errors.servers || '未配置',
      muted: !active,
    },
    {
      label: '隧道',
      value: data ? `${data.overview.runningTunnel}/${data.overview.tunnelCount} 运行中` : '未读取',
      muted: !data,
    },
    {
      label: '连接',
      value: data ? `${data.overview.currentConnection} 当前连接` : '未读取',
      muted: !data,
    },
    {
      label: '运行时在线',
      value: data ? formatDuration(data.overview.runtimeUptimeSeconds) : '未读取',
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
      detail: entry.failureReason || '连接失败',
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
  diagnosticRunMessage.value = '正在运行诊断...'
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
    diagnosticRunMessage.value = summary.message
    if (!options.silent) {
      toast[summary.tone](summary.message)
    }
  } catch (error) {
    const message = toErrorMessage(error)
    diagnosticRunTone.value = 'error'
    diagnosticRunMessage.value = `诊断失败：${message}`
    if (!options.silent) toast.error(diagnosticRunMessage.value)
  } finally {
    await waitForVisibleLoading(startedAt)
    loading.value = false
  }
}

async function loadLogs() {
  await logStore.refresh()
  if (logStore.status === 'error') {
    throw new Error(logStore.error || '日志加载失败')
  }
}

function buildRunSummary() {
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
      message: `诊断完成：${counts.error} 项失败，${counts.warning} 项警告。`,
    }
  }

  if (!selectedServer.value) {
    return {
      tone: 'warning' as const,
      message: '诊断完成：尚未配置服务器，已检查本机、运行时、隧道、证书和日志。',
    }
  }

  if (counts.warning) {
    return {
      tone: 'warning' as const,
      message: `诊断完成：${counts.ok} 项通过，${counts.warning} 项需要确认。`,
    }
  }

  return {
    tone: 'success' as const,
    message: `诊断完成：${counts.ok} 项全部通过。`,
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

async function copyDiagnosticReport(showToast = true) {
  if (!lastGeneratedAt.value && !loading.value) {
    await refreshDiagnostics()
  }
  await diagnosticsService.copyText(JSON.stringify(buildReportPayload(), null, 2))
  if (showToast) toast.success('诊断报告已复制')
}

async function copySystemInfo() {
  if (!systemInfo.value) {
    toast.warning('系统信息尚不可用')
    return
  }
  await diagnosticsService.copyText(JSON.stringify(systemInfo.value, null, 2))
  toast.success('系统信息已复制')
}

async function exportDebugBundle() {
  if (!lastGeneratedAt.value && !loading.value) {
    await refreshDiagnostics()
  }
  downloadJson(`gate-debug-bundle-${timestampForFilename()}.json`, buildReportPayload())
  toast.success('Debug Bundle 已导出')
}

async function openIssue() {
  await copyDiagnosticReport(false)
  await openUrl(GITHUB_ISSUE_URL)
  toast.success('诊断报告已复制，可粘贴到 Issue')
}

async function openLogDirectory() {
  await openDirectory(systemInfo.value?.logDir, '日志目录')
}

async function openConfigDirectory() {
  await openDirectory(systemInfo.value?.configDir, '配置目录')
}

function clearSupportCache() {
  diagnosticsService.clearSupportCache()
  refreshMemory()
  toast.success('帮助中心缓存已清理')
}

async function exportLogs() {
  if (!logStore.logs.length) {
    await capture('logs', loadLogs)
  }
  downloadLogs(logStore.logs, 'txt')
  toast.success('日志已导出')
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
  if (status === 'healthy') return '健康'
  if (status === 'warning') return '警告'
  if (status === 'critical') return '严重'
  return '离线'
}

function isSevereCertificateStatus(status: CertificateStatus) {
  return status === 'expired' || status === 'revoked' || status === 'deleted' || status === 'failed'
}

async function openDirectory(path: string | undefined, label: string) {
  if (!path) {
    toast.warning(`${label}尚不可用`)
    return
  }
  try {
    await openExternal(path)
  } catch {
    await diagnosticsService.copyText(path)
    toast.warning(`${label}无法直接打开，路径已复制`)
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
  if (!timestamp) return '未知'
  return new Date(timestamp).toLocaleString()
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
