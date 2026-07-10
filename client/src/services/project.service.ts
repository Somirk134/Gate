import { isTauri } from '@tauri-apps/api/core'
import { i18n } from '@/i18n'
import { TauriIpcClient } from '@/ipc'
import type { DashboardData } from '@/monitoring/types'
import type { CertificateListResponse } from '@views/certificates/types'
import type {
  ProjectDeleteImpact,
  ProjectDeleteMode,
  ProjectDeleteResponse,
  ProjectEnvironment,
  ProjectFormData,
  ProjectLogEntry,
  ProjectRecord,
  ProjectTemplate,
  ProjectTemplateProfile,
  TunnelRecommendation,
} from '@views/projects/types'
import { PROJECT_TEMPLATES } from '@views/projects/utils'

const ipc = new TauriIpcClient()
const LOCAL_KEY = 'gate.project.workspace.records'

function t(key: string, params?: Record<string, unknown>): string {
  return (i18n.global as unknown as { t: (key: string, params?: Record<string, unknown>) => string }).t(
    key,
    params,
  )
}

type ProjectCreatePayload = {
  name: string
  description?: string
  icon?: string
  color?: string
  template?: ProjectTemplate
  tunnelIds?: string[]
  domains?: string[]
  certificateIds?: string[]
  tags?: string[]
  environments?: ProjectEnvironment[]
  autoStart?: boolean
  startupPolicy?: string | null
}

type ProjectUpdatePayload = Partial<ProjectCreatePayload> & {
  favorite?: boolean
  pinned?: boolean
  notes?: ProjectRecord['notes']
}

function isTauriRuntime() {
  return isTauri()
}

export const projectService = {
  async list(): Promise<ProjectRecord[]> {
    if (isTauriRuntime()) {
      return ipc.invoke<ProjectRecord[]>('project_list')
    }
    return readLocalProjects()
  },

  async detail(projectId: string): Promise<ProjectRecord> {
    if (isTauriRuntime()) {
      return ipc.invoke<ProjectRecord>('project_detail', { projectId })
    }
    const project = readLocalProjects().find((item) => item.id === projectId)
    if (!project) throw new Error(t('project.errors.notFound'))
    return project
  },

  async create(form: ProjectFormData | ProjectCreatePayload): Promise<ProjectRecord> {
    const request = toCreatePayload(form)
    if (isTauriRuntime()) {
      return ipc.invoke<ProjectRecord>('project_create', { request })
    }
    const project = localProject(request)
    writeLocalProjects([project, ...readLocalProjects()])
    return project
  },

  async update(projectId: string, patch: ProjectUpdatePayload): Promise<ProjectRecord> {
    if (isTauriRuntime()) {
      return ipc.invoke<ProjectRecord>('project_update', { projectId, patch })
    }
    const projects = readLocalProjects()
    const index = projects.findIndex((item) => item.id === projectId)
    if (index === -1) throw new Error(t('project.errors.notFound'))
    projects[index] = {
      ...projects[index],
      ...patch,
      updatedAt: Date.now(),
      lastActivityAt: Date.now(),
    } as ProjectRecord
    writeLocalProjects(projects)
    return projects[index]
  },

  async deleteImpact(projectId: string): Promise<ProjectDeleteImpact> {
    if (isTauriRuntime()) {
      return ipc.invoke<ProjectDeleteImpact>('project_delete_impact', { projectId })
    }
    const project = await this.detail(projectId)
    const tunnelCount = project.tunnelIds.length
    const domainCount = project.domains.length
    const certificateCount = project.certificateIds.length
    return {
      tunnelCount,
      domainCount,
      certificateCount,
      hasReferences: tunnelCount > 0 || domainCount > 0 || certificateCount > 0,
    }
  },

  async delete(projectId: string, mode: ProjectDeleteMode): Promise<ProjectDeleteResponse> {
    if (isTauriRuntime()) {
      return ipc.invoke<ProjectDeleteResponse>('project_delete', { projectId, mode })
    }
    const impact = await this.deleteImpact(projectId)
    const projects = readLocalProjects().filter((item) => item.id !== projectId)
    writeLocalProjects(projects)
    return {
      projectId,
      impact,
      deletedTunnelIds: [],
      failedTunnelIds: [],
    }
  },

  async setFavorite(projectId: string, favorite: boolean): Promise<ProjectRecord> {
    if (isTauriRuntime()) {
      return ipc.invoke<ProjectRecord>('project_set_favorite', { projectId, favorite })
    }
    return this.update(projectId, { favorite })
  },

  async setPinned(projectId: string, pinned: boolean): Promise<ProjectRecord> {
    if (isTauriRuntime()) {
      return ipc.invoke<ProjectRecord>('project_set_pinned', { projectId, pinned })
    }
    return this.update(projectId, { pinned })
  },

  async addTunnel(projectId: string, tunnelId: string): Promise<ProjectRecord> {
    if (isTauriRuntime()) {
      return ipc.invoke<ProjectRecord>('project_add_tunnel', { projectId, tunnelId })
    }
    const project = await this.detail(projectId)
    return this.update(projectId, {
      tunnelIds: [...new Set([...project.tunnelIds, tunnelId])],
    })
  },

  async removeTunnel(projectId: string, tunnelId: string): Promise<ProjectRecord> {
    if (isTauriRuntime()) {
      return ipc.invoke<ProjectRecord>('project_remove_tunnel', { projectId, tunnelId })
    }
    const project = await this.detail(projectId)
    return this.update(projectId, {
      tunnelIds: project.tunnelIds.filter((item) => item !== tunnelId),
    })
  },

  async moveTunnel(sourceProjectId: string, targetProjectId: string, tunnelId: string) {
    if (isTauriRuntime()) {
      return ipc.invoke<[ProjectRecord, ProjectRecord]>('project_move_tunnel', {
        sourceProjectId,
        targetProjectId,
        tunnelId,
      })
    }
    await this.removeTunnel(sourceProjectId, tunnelId)
    const target = await this.addTunnel(targetProjectId, tunnelId)
    const source = await this.detail(sourceProjectId)
    return [source, target] as [ProjectRecord, ProjectRecord]
  },

  async addDomain(projectId: string, domain: string): Promise<ProjectRecord> {
    if (isTauriRuntime()) {
      return ipc.invoke<ProjectRecord>('project_add_domain', { projectId, domain })
    }
    const project = await this.detail(projectId)
    return this.update(projectId, {
      domains: [...new Set([...project.domains, domain])],
    })
  },

  async removeDomain(projectId: string, domain: string): Promise<ProjectRecord> {
    if (isTauriRuntime()) {
      return ipc.invoke<ProjectRecord>('project_remove_domain', { projectId, domain })
    }
    const project = await this.detail(projectId)
    return this.update(projectId, {
      domains: project.domains.filter((item) => item !== domain),
    })
  },

  async addCertificate(projectId: string, certificateId: string): Promise<ProjectRecord> {
    if (isTauriRuntime()) {
      return ipc.invoke<ProjectRecord>('project_add_certificate', { projectId, certificateId })
    }
    const project = await this.detail(projectId)
    return this.update(projectId, {
      certificateIds: [...new Set([...project.certificateIds, certificateId])],
    })
  },

  async removeCertificate(projectId: string, certificateId: string): Promise<ProjectRecord> {
    if (isTauriRuntime()) {
      return ipc.invoke<ProjectRecord>('project_remove_certificate', { projectId, certificateId })
    }
    const project = await this.detail(projectId)
    return this.update(projectId, {
      certificateIds: project.certificateIds.filter((item) => item !== certificateId),
    })
  },

  async templates(): Promise<ProjectTemplateProfile[]> {
    if (isTauriRuntime()) {
      return ipc.invoke<ProjectTemplateProfile[]>('project_templates')
    }
    return PROJECT_TEMPLATES
  },

  async recommendTunnels(template: ProjectTemplate): Promise<TunnelRecommendation[]> {
    if (isTauriRuntime()) {
      return ipc.invoke<TunnelRecommendation[]>('project_recommend_tunnels', { template })
    }
    return PROJECT_TEMPLATES.find((item) => item.key === template)?.recommendations ?? []
  },

  async start(projectId: string): Promise<{ startedTunnelIds: string[]; failedTunnelIds: [string, string][] }> {
    if (isTauriRuntime()) {
      return ipc.invoke('project_start', { projectId })
    }
    // 非 Tauri 环境无实际运行时，返回空
    return { startedTunnelIds: [], failedTunnelIds: [] }
  },

  async stop(projectId: string): Promise<{ startedTunnelIds: string[]; failedTunnelIds: [string, string][] }> {
    if (isTauriRuntime()) {
      return ipc.invoke('project_stop', { projectId })
    }
    return { startedTunnelIds: [], failedTunnelIds: [] }
  },

  async dashboard(): Promise<DashboardData> {
    if (isTauriRuntime()) {
      return ipc.invoke<DashboardData>('runtime_get_dashboard')
    }
    return emptyDashboard()
  },

  async logs(): Promise<ProjectLogEntry[]> {
    if (isTauriRuntime()) {
      return ipc.invoke<ProjectLogEntry[]>('runtime_get_logs')
    }
    return []
  },

  async certificates(): Promise<CertificateListResponse> {
    if (isTauriRuntime()) {
      return ipc.invoke<CertificateListResponse>('certificate_list')
    }
    return {
      storeRoot: '',
      certificates: [],
      generatedAt: Date.now(),
    }
  },
}

function toCreatePayload(form: ProjectFormData | ProjectCreatePayload): ProjectCreatePayload {
  return {
    name: form.name,
    description: form.description,
    icon: form.icon,
    color: form.color,
    template: form.template ?? 'blank',
    tags: form.tags ?? [],
    environments: 'environments' in form ? form.environments : [],
    autoStart: 'autoStart' in form ? form.autoStart : false,
    startupPolicy: 'startupPolicy' in form ? form.startupPolicy : null,
    tunnelIds: 'tunnelIds' in form ? form.tunnelIds : [],
    domains: 'domains' in form ? form.domains : [],
    certificateIds: 'certificateIds' in form ? form.certificateIds : [],
  }
}

function localProject(payload: ProjectCreatePayload): ProjectRecord {
  const now = Date.now()
  return {
    id: crypto.randomUUID?.() ?? `project-${now}`,
    name: payload.name.trim(),
    description: payload.description ?? '',
    icon: payload.icon ?? 'package',
    color: (payload.color ?? 'blue') as ProjectRecord['color'],
    template: payload.template ?? 'blank',
    tunnelIds: payload.tunnelIds ?? [],
    domains: payload.domains ?? [],
    certificateIds: payload.certificateIds ?? [],
    tags: payload.tags ?? [],
    environments: payload.environments ?? [],
    notes: [],
    favorite: false,
    pinned: false,
    autoStart: payload.autoStart ?? false,
    startupPolicy: payload.startupPolicy,
    lastActivityAt: now,
    createdAt: now,
    updatedAt: now,
  }
}

function readLocalProjects(): ProjectRecord[] {
  try {
    const raw = localStorage.getItem(LOCAL_KEY)
    if (!raw) return []
    const records = JSON.parse(raw)
    return Array.isArray(records) ? records : []
  } catch {
    return []
  }
}

function writeLocalProjects(projects: ProjectRecord[]) {
  localStorage.setItem(LOCAL_KEY, JSON.stringify(projects))
}

function emptyDashboard(): DashboardData {
  const now = Date.now()
  return {
    overview: {
      tunnelCount: 0,
      runningTunnel: 0,
      currentConnection: 0,
      todayTraffic: 0,
      totalTraffic: 0,
      averageRttMs: 0,
      runtimeUptimeSeconds: 0,
      healthScore: 0,
    },
    statistics: {
      collectedAt: now,
      tunnel: {
        tunnelCount: 0,
        runningTunnel: 0,
        stoppedTunnel: 0,
        upload: 0,
        download: 0,
        peakSpeedBps: 0,
        averageSpeedBps: 0,
        runningTimeSeconds: 0,
        todayTraffic: 0,
        totalTraffic: 0,
      },
      traffic: {
        uploadBytes: 0,
        downloadBytes: 0,
        uploadSpeedBps: 0,
        downloadSpeedBps: 0,
        peakSpeedBps: 0,
        averageSpeedBps: 0,
        todayTrafficBytes: 0,
        totalTrafficBytes: 0,
      },
      connection: {
        currentConnection: 0,
        totalConnection: 0,
        success: 0,
        failure: 0,
        reconnect: 0,
        disconnect: 0,
        connectionDurationMs: 0,
        averageRttMs: 0,
      },
      runtime: {
        runningTask: 0,
        workerCount: 0,
        schedulerQueue: 0,
        bufferUsage: 0,
        sessionCount: 0,
        runtimeUptimeSeconds: 0,
      },
      tls: {
        sessionCount: 0,
        handshakeCount: 0,
        errorCount: 0,
        trafficBytes: 0,
      },
      system: {
        cpuUsage: 0,
        memoryUsage: 0,
        threadCount: 0,
        processUptimeSeconds: 0,
      },
      client: {
        onlineTimeSeconds: 0,
        openProject: 0,
        currentWorkspace: t('project.previewWorkspace'),
        memoryBytes: 0,
      },
    },
    realtimeSpeed: [],
    connectionTrend: [],
    trafficTrend: [],
    tunnelStatus: [],
    serverStatus: [],
    systemHealth: {
      overall: 'offline',
      signals: [],
      updatedAt: now,
    },
    tunnels: [],
    recentActivity: [],
    generatedAt: now,
  }
}
