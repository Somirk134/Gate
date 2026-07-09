import { TauriIpcClient } from '@/ipc'

const ipc = new TauriIpcClient()

const RECENT_SERVERS_KEY = 'gate.recentServers'
const CONNECTION_HISTORY_KEY = 'gate.connectionHistory'

export type DiagnosticStatus = 'ok' | 'warning' | 'error'

export interface DiagnosticAction {
  label: string
  description: string
}

export interface DiagnosticFinding {
  id: string
  label: string
  status: DiagnosticStatus
  reason: string
  possibleCause: string
  solution: string
  elapsedMs?: number
}

export interface ConnectionTestReport {
  ok: boolean
  code: string
  title: string
  reason: string
  possibleCause: string
  solution: string
  elapsedMs: number
  checkedAt: number
  actions: DiagnosticAction[]
}

export interface DeploymentCheckReport {
  ok: boolean
  checkedAt: number
  summary: string
  findings: DiagnosticFinding[]
}

export interface SystemInfoReport {
  clientVersion: string
  serverVersion: string
  protocolVersion: string
  rustVersion: string
  os: string
  arch: string
  cpu: string
  memory: string
  configDir: string
  logDir: string
  currentDir: string
}

export interface RecentServer {
  serverAddr: string
  lastConnectedAt: number
  favorite: boolean
  successCount: number
}

export interface ConnectionHistoryEntry {
  id: string
  serverAddr: string
  connectedAt: number
  result: 'success' | 'failed'
  failureReason: string
  elapsedMs: number
}

export interface ServerConnectionInput {
  host: string
  port: number
  token: string
}

export const diagnosticsService = {
  formatServerAddr(input: Pick<ServerConnectionInput, 'host' | 'port'>) {
    return `${input.host.trim()}:${input.port}`
  },

  async testConnection(input: ServerConnectionInput): Promise<ConnectionTestReport> {
    const serverAddr = this.formatServerAddr(input)
    try {
      const report = await ipc.invoke<ConnectionTestReport>('diagnostics_test_connection', {
        serverAddr,
        token: input.token,
        timeoutMs: 5000,
      })
      saveHistory(serverAddr, report)
      if (report.ok) saveRecentServer(serverAddr)
      return report
    } catch (error) {
      const report = fallbackConnectionReport(serverAddr, input, error)
      saveHistory(serverAddr, report)
      return report
    }
  },

  async runDeployment(serverAddr?: string): Promise<DeploymentCheckReport> {
    // 帮助中心需要真实部署检查结果，IPC 失败交给页面展示为不可用。
    return ipc.invoke<DeploymentCheckReport>('diagnostics_run_deployment', {
      serverAddr,
    })
  },

  async collectSystemInfo(): Promise<SystemInfoReport> {
    // 系统状态禁止使用 Mock，必须直接读取桌面运行时返回的数据。
    return ipc.invoke<SystemInfoReport>('diagnostics_collect_system_info')
  },

  getRecentServers(): RecentServer[] {
    return readJson<RecentServer[]>(RECENT_SERVERS_KEY, [])
  },

  getConnectionHistory(): ConnectionHistoryEntry[] {
    return readJson<ConnectionHistoryEntry[]>(CONNECTION_HISTORY_KEY, [])
  },

  clearHistory() {
    localStorage.removeItem(CONNECTION_HISTORY_KEY)
  },

  clearSupportCache() {
    localStorage.removeItem(RECENT_SERVERS_KEY)
    localStorage.removeItem(CONNECTION_HISTORY_KEY)
  },

  copyText(text: string) {
    return navigator.clipboard?.writeText(text)
  },
}

function fallbackConnectionReport(
  serverAddr: string,
  input: ServerConnectionInput,
  error: unknown,
): ConnectionTestReport {
  const startedAt = Date.now()
  if (!input.host.trim()) {
    return makeConnectionReport(
      false,
      'ADDRESS_INVALID',
      '服务器地址为空',
      '还没有填写服务器地址。',
      '首次配置时容易只填写端口或 Token。',
      '填写服务器域名或 IP，再点击测试连接。',
      0,
    )
  }
  if (!input.token.trim()) {
    return makeConnectionReport(
      false,
      'TOKEN_EMPTY',
      'Token 为空',
      '还没有填写服务端 Token。',
      'Token 未复制、复制漏掉或服务端尚未生成 Token。',
      '从服务端配置复制完整 Token 后重试。',
      0,
    )
  }
  return makeConnectionReport(
    false,
    'CLIENT_RUNTIME_UNAVAILABLE',
    '无法调用桌面诊断运行时',
    error instanceof Error ? error.message : `无法测试 ${serverAddr}`,
    '当前可能运行在浏览器预览模式，或 Tauri IPC 尚未就绪。',
    '在 Gate Desktop 中重新测试；预览模式下只能查看表单和引导流程。',
    Date.now() - startedAt,
  )
}

function makeConnectionReport(
  ok: boolean,
  code: string,
  title: string,
  reason: string,
  possibleCause: string,
  solution: string,
  elapsedMs: number,
): ConnectionTestReport {
  return {
    ok,
    code,
    title,
    reason,
    possibleCause,
    solution,
    elapsedMs,
    checkedAt: Date.now(),
    actions: [
      { label: '查看日志', description: '跳转到日志页面查看客户端日志。' },
      { label: '复制错误', description: '复制结构化错误内容。' },
    ],
  }
}

function saveRecentServer(serverAddr: string) {
  const list = readJson<RecentServer[]>(RECENT_SERVERS_KEY, [])
  const current = list.find((server) => server.serverAddr === serverAddr)
  if (current) {
    current.lastConnectedAt = Date.now()
    current.successCount += 1
  } else {
    list.unshift({
      serverAddr,
      lastConnectedAt: Date.now(),
      favorite: false,
      successCount: 1,
    })
  }
  localStorage.setItem(RECENT_SERVERS_KEY, JSON.stringify(list.slice(0, 6)))
}

function saveHistory(serverAddr: string, report: ConnectionTestReport) {
  const list = readJson<ConnectionHistoryEntry[]>(CONNECTION_HISTORY_KEY, [])
  list.unshift({
    id: `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
    serverAddr,
    connectedAt: Date.now(),
    result: report.ok ? 'success' : 'failed',
    failureReason: report.ok ? '' : report.title,
    elapsedMs: report.elapsedMs,
  })
  localStorage.setItem(CONNECTION_HISTORY_KEY, JSON.stringify(list.slice(0, 10)))
}

function readJson<T>(key: string, fallback: T): T {
  try {
    const value = localStorage.getItem(key)
    return value ? (JSON.parse(value) as T) : fallback
  } catch {
    return fallback
  }
}
