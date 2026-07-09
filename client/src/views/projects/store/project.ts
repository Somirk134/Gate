import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { i18n } from '@/i18n'
import { projectService } from '@/services/project.service'
import type { DashboardData, DashboardTunnel } from '@/monitoring/types'
import type { CertificateSummary } from '@views/certificates/types'
import type {
  Project,
  ProjectDeleteMode,
  ProjectFormData,
  ProjectLoadStatus,
  ProjectLogEntry,
  ProjectRecord,
  ProjectTemplateProfile,
} from '../types'
import { PROJECT_TEMPLATES, formatRelativeTime } from '../utils'

function t(key: string, params?: Record<string, unknown>): string {
  return (i18n.global as unknown as { t: (key: string, params?: Record<string, unknown>) => string }).t(
    key,
    params,
  )
}

function currentLocale(): string {
  const locale = (i18n.global as unknown as { locale: string | { value: string } }).locale
  return typeof locale === 'string' ? locale : locale.value
}

export const useProjectStore = defineStore('project', () => {
  const records = ref<ProjectRecord[]>([])
  const projects = ref<Project[]>([])
  const templates = ref<ProjectTemplateProfile[]>(PROJECT_TEMPLATES)
  const dashboard = ref<DashboardData | null>(null)
  const logs = ref<ProjectLogEntry[]>([])
  const certificates = ref<CertificateSummary[]>([])
  const status = ref<ProjectLoadStatus>('idle')
  const error = ref<string>('')
  const lastUpdated = ref<number>(0)
  const serverNames = ref<string[]>([t('project.defaultWorkspace')])

  const isLoading = computed(() => status.value === 'loading')
  const isError = computed(() => status.value === 'error')
  const isReady = computed(() => status.value === 'success')
  const hasProjects = computed(() => projects.value.length > 0)
  const pinnedProjects = computed(() => projects.value.filter((p) => p.pinned))
  const favoriteProjects = computed(() => projects.value.filter((p) => p.favorite))
  const runningProjects = computed(() => projects.value.filter((p) => p.status !== 'stopped'))
  const stoppedProjects = computed(() => projects.value.filter((p) => p.status === 'stopped'))
  const recentProjects = computed(() =>
    [...projects.value].sort((a, b) => b.lastActivityAt - a.lastActivityAt).slice(0, 8),
  )
  const totalTunnels = computed(() =>
    projects.value.reduce((sum, project) => sum + project.tunnelCount, 0),
  )
  const runningTunnelCount = computed(() =>
    projects.value.reduce((sum, project) => sum + project.runningTunnelCount, 0),
  )

  function getById(id: string): Project | undefined {
    return projects.value.find((p) => p.id === id)
  }

  async function load(): Promise<void> {
    status.value = 'loading'
    error.value = ''
    try {
      const [projectRows, runtimeDashboard, runtimeLogs, certificateList, templateRows] =
        await Promise.all([
          projectService.list(),
          projectService.dashboard(),
          projectService.logs(),
          projectService.certificates(),
          projectService.templates().catch(() => PROJECT_TEMPLATES),
        ])

      records.value = projectRows
      dashboard.value = runtimeDashboard
      logs.value = runtimeLogs
      certificates.value = certificateList.certificates
      templates.value = templateRows.length ? templateRows : PROJECT_TEMPLATES
      serverNames.value = [
        runtimeDashboard.statistics.client.currentWorkspace || t('project.defaultWorkspace'),
      ]
      projects.value = projectRows.map((record) =>
        mapProject(record, runtimeDashboard, runtimeLogs, certificateList.certificates),
      )
      status.value = 'success'
      lastUpdated.value = Date.now()
    } catch (e) {
      status.value = 'error'
      error.value = e instanceof Error ? e.message : t('project.loadFailed')
    }
  }

  async function refresh(): Promise<void> {
    return load()
  }

  async function createProject(form: ProjectFormData): Promise<Project> {
    const record = await projectService.create(form)
    records.value = [record, ...records.value.filter((item) => item.id !== record.id)]
    await refresh()
    const created = getById(record.id)
    if (!created) throw new Error(t('project.savedReloadFailed'))
    return created
  }

  async function createDefaultProject(): Promise<Project> {
    return createProject({
      name: t('project.defaultName'),
      icon: 'projects',
      color: 'blue',
      template: 'blank',
      description: t('project.defaultDescription'),
      serverName: t('project.defaultWorkspace'),
      autoStart: false,
      tags: ['default'],
      remark: '',
      environments: [],
      startupPolicy: null,
    })
  }

  async function updateProject(id: string, patch: Partial<ProjectFormData>): Promise<void> {
    await projectService.update(id, patch)
    await refresh()
  }

  async function removeProject(id: string, mode: ProjectDeleteMode = 'projectOnly'): Promise<void> {
    await projectService.delete(id, mode)
    await refresh()
  }

  async function addTunnel(projectId: string, tunnelId: string): Promise<void> {
    await projectService.addTunnel(projectId, tunnelId)
    await refresh()
  }

  async function removeTunnel(projectId: string, tunnelId: string): Promise<void> {
    await projectService.removeTunnel(projectId, tunnelId)
    await refresh()
  }

  async function moveTunnel(
    sourceProjectId: string,
    targetProjectId: string,
    tunnelId: string,
  ): Promise<void> {
    await projectService.moveTunnel(sourceProjectId, targetProjectId, tunnelId)
    await refresh()
  }

  async function addDomain(projectId: string, domain: string): Promise<void> {
    await projectService.addDomain(projectId, domain)
    await refresh()
  }

  async function removeDomain(projectId: string, domain: string): Promise<void> {
    await projectService.removeDomain(projectId, domain)
    await refresh()
  }

  async function addCertificate(projectId: string, certificateId: string): Promise<void> {
    await projectService.addCertificate(projectId, certificateId)
    await refresh()
  }

  async function removeCertificate(projectId: string, certificateId: string): Promise<void> {
    await projectService.removeCertificate(projectId, certificateId)
    await refresh()
  }

  async function startProject(id: string): Promise<void> {
    await projectService.update(id, {})
    await refresh()
  }

  async function stopProject(id: string): Promise<void> {
    await projectService.update(id, {})
    await refresh()
  }

  async function togglePin(id: string): Promise<void> {
    const project = getById(id)
    if (!project) return
    await projectService.setPinned(id, !project.pinned)
    await refresh()
  }

  async function toggleFavorite(id: string): Promise<void> {
    const project = getById(id)
    if (!project) return
    await projectService.setFavorite(id, !project.favorite)
    await refresh()
  }

  return {
    records,
    projects,
    templates,
    dashboard,
    logs,
    certificates,
    status,
    error,
    lastUpdated,
    serverNames,
    isLoading,
    isError,
    isReady,
    hasProjects,
    pinnedProjects,
    favoriteProjects,
    runningProjects,
    stoppedProjects,
    recentProjects,
    totalTunnels,
    runningTunnelCount,
    getById,
    load,
    refresh,
    createProject,
    createDefaultProject,
    updateProject,
    removeProject,
    addTunnel,
    removeTunnel,
    moveTunnel,
    addDomain,
    removeDomain,
    addCertificate,
    removeCertificate,
    startProject,
    stopProject,
    togglePin,
    toggleFavorite,
  }
})

function mapProject(
  record: ProjectRecord,
  dashboard: DashboardData,
  logs: ProjectLogEntry[],
  certificates: CertificateSummary[],
): Project {
  const tunnelIds = new Set(record.tunnelIds)
  const projectTunnels = dashboard.tunnels.filter((tunnel) => tunnelIds.has(tunnel.id))
  const discoveredDomains = projectTunnels
    .map((tunnel) => tunnel.host)
    .filter((value): value is string => Boolean(value))
  const domains = unique([...record.domains, ...discoveredDomains])
  const certificateIds = unique([
    ...record.certificateIds,
    ...certificates
      .filter((certificate) => domains.includes(certificate.domain))
      .map((certificate) => certificate.domain),
  ])
  const projectLogs = logs
    .filter((log) => Boolean(log.tunnelId) && tunnelIds.has(log.tunnelId as string))
    .sort((a, b) => b.timestamp - a.timestamp)
  const recentLogs = projectLogs.slice(0, 10)
  const recentErrors = projectLogs.filter((log) => log.level === 'error').slice(0, 6)
  const runningTunnelCount = projectTunnels.filter((tunnel) => tunnel.status === 'running').length
  const warningTunnelCount = projectTunnels.filter((tunnel) => tunnel.status === 'warning').length
  const status =
    warningTunnelCount > 0
      ? 'error'
      : projectTunnels.length === 0 || runningTunnelCount === 0
        ? 'stopped'
        : runningTunnelCount === projectTunnels.length
          ? 'running'
          : 'partial'
  const trafficBytes = projectTunnels.reduce((sum, tunnel) => sum + (tunnel.trafficBytes ?? 0), 0)
  const requestCount = projectTunnels.reduce((sum, tunnel) => sum + (tunnel.requestCount ?? 0), 0)
  const latencySamples = projectTunnels
    .map((tunnel) => tunnel.averageResponseTimeMs ?? 0)
    .filter((value) => value > 0)
  const averageLatencyMs = latencySamples.length
    ? latencySamples.reduce((sum, value) => sum + value, 0) / latencySamples.length
    : 0
  const tlsSessionCount = projectTunnels.reduce(
    (sum, tunnel) => sum + (tunnel.tls?.sessionCount ?? 0),
    0,
  )
  const bandwidthBps = projectTunnels.reduce(
    (sum, tunnel) => sum + tunnel.uploadSpeedBps + tunnel.downloadSpeedBps,
    0,
  )
  const projectCertificates = certificates.filter(
    (certificate) =>
      certificateIds.includes(certificate.domain) ||
      certificateIds.includes(certificate.fingerprintSha256),
  )

  return {
    id: record.id,
    name: record.name,
    description: record.description || t('project.descriptionFallback'),
    icon: record.icon || 'package',
    color: record.color || 'blue',
    template: record.template,
    tags: record.tags,
    serverName: t('project.defaultWorkspace'),
    autoStart: record.autoStart,
    startupPolicy: record.startupPolicy,
    remark: record.notes[0]?.content,
    status,
    pinned: record.pinned,
    favorite: record.favorite,
    lastUsedAt: record.lastActivityAt,
    lastActivityAt: record.lastActivityAt,
    tunnelIds: record.tunnelIds,
    domains,
    certificateIds,
    environments: record.environments,
    notes: record.notes,
    tunnelCount: projectTunnels.length,
    runningTunnelCount,
    domainCount: domains.length,
    certificateCount: certificateIds.length,
    statistics: {
      todayTraffic: trafficBytes,
      totalTraffic: trafficBytes,
      uptime: projectTunnels.reduce((sum, tunnel) => sum + tunnel.uptimeSeconds, 0),
      connections: projectTunnels.reduce((sum, tunnel) => sum + tunnel.connections, 0),
      tunnelCount: projectTunnels.length,
      runningTunnelCount,
      httpsCount: projectTunnels.filter((tunnel) => tunnel.protocol === 'https').length,
      tcpCount: projectTunnels.filter((tunnel) => tunnel.protocol === 'tcp').length,
      requestCount,
      tlsSessionCount,
      errorCount: recentErrors.length,
      averageLatencyMs,
      bandwidthBps,
      cpuUsage: dashboard.statistics.system.cpuUsage,
      memoryUsage: dashboard.statistics.system.memoryUsage,
    },
    recentLogs,
    recentErrors,
    certificateStatus: certificateStatus(domains, projectCertificates),
    lastStartedAt: formatRelativeTime(record.lastActivityAt, t, currentLocale()),
    createdAt: record.createdAt,
    updatedAt: record.updatedAt,
  }
}

function certificateStatus(
  domains: string[],
  certificates: CertificateSummary[],
): Project['certificateStatus'] {
  if (!domains.length) return 'unknown'
  if (!certificates.length) return 'missing'
  if (certificates.some((certificate) => certificate.daysRemaining <= 14)) return 'warning'
  return 'healthy'
}

function unique(values: string[]) {
  return [...new Set(values.map((value) => value.trim()).filter(Boolean))]
}

export function projectTunnels(project: Project, runtimeDashboard: DashboardData | null) {
  if (!runtimeDashboard) return []
  const ids = new Set(project.tunnelIds)
  return runtimeDashboard.tunnels.filter((tunnel: DashboardTunnel) => ids.has(tunnel.id))
}
