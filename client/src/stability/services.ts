import type {
  ConnectionHealth,
  HealthReport,
  HealthSignal,
  HealthStatus,
  HeartbeatConfig,
  HeartbeatMetrics,
  HeartbeatSnapshot,
  HeartbeatState,
  ReconnectPolicy,
  ReconnectRequest,
  ReconnectSnapshot,
  RecoveryContext,
  RecoveryResult,
  StateSnapshot,
  StateSyncResult,
  SyncConfig,
  SyncTarget,
} from './types'

const defaultHeartbeatConfig = (): HeartbeatConfig => ({
  intervalMs: 10_000,
  timeoutMs: 15_000,
  retryCount: 3,
  retryDelayMs: 2_000,
  maxMissedHeartbeat: 3,
})

const defaultReconnectPolicy = (): ReconnectPolicy => ({
  kind: 'exponential_backoff',
  baseDelayMs: 500,
  maxDelayMs: 30_000,
  factor: 2,
  maxAttempts: 10,
})

const defaultSyncConfig = (): SyncConfig => ({
  intervalMs: 5_000,
  batchSize: 1_024,
  includeTunnelState: true,
  includeProjectState: true,
  includeServerState: true,
  includeConfiguration: true,
  includeStatistics: true,
  includeLogCursor: false,
})

const emptyHeartbeatMetrics = (): HeartbeatMetrics => ({
  pingCount: 0,
  pongCount: 0,
  timeoutCount: 0,
  retryCount: 0,
  heartbeatCount: 0,
  averageRttMs: 0,
})

/** Promise-based heartbeat state machine. */
export class HeartbeatService {
  private readonly snapshots = new Map<string, HeartbeatSnapshot>()
  private readonly pingStartedAt = new Map<string, number>()

  constructor(private readonly config: HeartbeatConfig = defaultHeartbeatConfig()) {}

  /** Start heartbeat tracking for a tunnel. */
  async start(tunnelId: string): Promise<HeartbeatSnapshot> {
    const snapshot = this.ensure(tunnelId)
    snapshot.state = 'running'
    snapshot.missedHeartbeat = 0
    snapshot.retryAttempt = 0
    return { ...snapshot, metrics: { ...snapshot.metrics } }
  }

  /** Stop heartbeat tracking for a tunnel. */
  async stop(tunnelId: string): Promise<HeartbeatSnapshot> {
    const snapshot = this.ensure(tunnelId)
    snapshot.state = 'stopped'
    return { ...snapshot, metrics: { ...snapshot.metrics } }
  }

  /** Pause heartbeat tracking without removing its snapshot. */
  async pause(tunnelId: string): Promise<HeartbeatSnapshot> {
    const snapshot = this.ensure(tunnelId)
    snapshot.state = 'idle'
    return { ...snapshot, metrics: { ...snapshot.metrics } }
  }

  /** Resume a paused heartbeat. */
  async resume(tunnelId: string): Promise<HeartbeatSnapshot> {
    const snapshot = this.ensure(tunnelId)
    snapshot.state = 'running'
    return { ...snapshot, metrics: { ...snapshot.metrics } }
  }

  /** Advance heartbeat state once. */
  async tick(tunnelId: string): Promise<HeartbeatSnapshot> {
    const snapshot = this.ensure(tunnelId)

    if (snapshot.state === 'running') {
      return this.ping(tunnelId)
    }

    if (snapshot.state === 'waiting_pong' && this.isTimedOut(snapshot)) {
      return this.timeout(tunnelId)
    }

    if (snapshot.state === 'timeout' && snapshot.retryAttempt < this.config.retryCount) {
      snapshot.state = 'retrying'
      snapshot.retryAttempt += 1
      snapshot.metrics.retryCount += 1
      return this.ping(tunnelId)
    }

    return { ...snapshot, metrics: { ...snapshot.metrics } }
  }

  /** Record that a ping should be sent by the transport layer. */
  async ping(tunnelId: string): Promise<HeartbeatSnapshot> {
    const snapshot = this.ensure(tunnelId)
    snapshot.state = 'waiting_pong'
    snapshot.sequence += 1
    snapshot.lastPingAt = Date.now()
    snapshot.metrics.pingCount += 1
    snapshot.metrics.heartbeatCount += 1
    this.pingStartedAt.set(tunnelId, snapshot.lastPingAt)
    return { ...snapshot, metrics: { ...snapshot.metrics } }
  }

  /** Record a pong received by the transport layer. */
  async pong(tunnelId: string, _sequence?: number): Promise<HeartbeatSnapshot> {
    const snapshot = this.ensure(tunnelId)
    const startedAt = this.pingStartedAt.get(tunnelId) ?? Date.now()
    const rttMs = Math.max(0, Date.now() - startedAt)

    snapshot.state = 'running'
    snapshot.missedHeartbeat = 0
    snapshot.retryAttempt = 0
    snapshot.lastPongAt = Date.now()
    snapshot.metrics.pongCount += 1
    snapshot.metrics.lastRttMs = rttMs
    snapshot.metrics.averageRttMs =
      (snapshot.metrics.averageRttMs * (snapshot.metrics.pongCount - 1) + rttMs) /
      snapshot.metrics.pongCount
    return { ...snapshot, metrics: { ...snapshot.metrics } }
  }

  /** Mark heartbeat timeout. */
  async timeout(tunnelId: string): Promise<HeartbeatSnapshot> {
    const snapshot = this.ensure(tunnelId)
    snapshot.state = 'timeout'
    snapshot.missedHeartbeat += 1
    snapshot.lastTimeoutAt = Date.now()
    snapshot.metrics.timeoutCount += 1
    return { ...snapshot, metrics: { ...snapshot.metrics } }
  }

  /** Return the latest heartbeat snapshot. */
  async snapshot(tunnelId: string): Promise<HeartbeatSnapshot | undefined> {
    const snapshot = this.snapshots.get(tunnelId)
    return snapshot ? { ...snapshot, metrics: { ...snapshot.metrics } } : undefined
  }

  private ensure(tunnelId: string): HeartbeatSnapshot {
    const existing = this.snapshots.get(tunnelId)
    if (existing) {
      return existing
    }

    const snapshot: HeartbeatSnapshot = {
      tunnelId,
      state: 'idle',
      sequence: 0,
      missedHeartbeat: 0,
      retryAttempt: 0,
      metrics: emptyHeartbeatMetrics(),
    }
    this.snapshots.set(tunnelId, snapshot)
    return snapshot
  }

  private isTimedOut(snapshot: HeartbeatSnapshot) {
    if (!snapshot.lastPingAt) {
      return false
    }

    return Date.now() - snapshot.lastPingAt >= this.config.timeoutMs
  }
}

/** Promise-based reconnect queue and scheduler. */
export class ReconnectService {
  private readonly queue: ReconnectRequest[] = []
  private readonly snapshots = new Map<string, ReconnectSnapshot>()

  constructor(private readonly policy: ReconnectPolicy = defaultReconnectPolicy()) {}

  /** Queue automatic reconnect work. */
  async autoReconnect(
    tunnelId: string,
    connectionId?: string,
    reason?: string,
  ): Promise<ReconnectSnapshot> {
    return this.enqueue(tunnelId, connectionId, 'auto', reason)
  }

  /** Queue manual reconnect work. */
  async manualReconnect(
    tunnelId: string,
    connectionId?: string,
    reason?: string,
  ): Promise<ReconnectSnapshot> {
    return this.enqueue(tunnelId, connectionId, 'manual', reason)
  }

  /** Schedule the next reconnect attempt without performing network I/O. */
  async scheduleNext(): Promise<ReconnectRequest | undefined> {
    const request = this.queue.shift()
    if (!request) {
      return undefined
    }

    request.attempt += 1
    request.scheduledDelayMs = this.delayForAttempt(request.attempt) ?? 0
    request.nextRetryAt = Date.now() + request.scheduledDelayMs

    const snapshot = this.snapshots.get(request.tunnelId)
    if (snapshot) {
      snapshot.state = 'reconnecting'
      snapshot.attempt = request.attempt
      snapshot.reconnectCount += 1
      snapshot.nextRetryAt = request.nextRetryAt
      snapshot.updatedAt = Date.now()
    }

    return { ...request }
  }

  /** Mark a reconnect attempt as successful. */
  async markSucceeded(tunnelId: string): Promise<ReconnectSnapshot | undefined> {
    const snapshot = this.snapshots.get(tunnelId)
    if (!snapshot) {
      return undefined
    }

    snapshot.state = 'succeeded'
    snapshot.lastError = undefined
    snapshot.nextRetryAt = undefined
    snapshot.updatedAt = Date.now()
    return { ...snapshot }
  }

  /** Mark a reconnect attempt as failed. */
  async markFailed(tunnelId: string, reason: string): Promise<ReconnectSnapshot | undefined> {
    const snapshot = this.snapshots.get(tunnelId)
    if (!snapshot) {
      return undefined
    }

    snapshot.state = 'failed'
    snapshot.failedCount += 1
    snapshot.lastError = reason
    snapshot.nextRetryAt = undefined
    snapshot.updatedAt = Date.now()
    return { ...snapshot }
  }

  /** Return the latest reconnect snapshot. */
  async snapshot(tunnelId: string): Promise<ReconnectSnapshot | undefined> {
    const snapshot = this.snapshots.get(tunnelId)
    return snapshot ? { ...snapshot } : undefined
  }

  /** Return queued reconnect work count. */
  async queueLength(): Promise<number> {
    return this.queue.length
  }

  private enqueue(
    tunnelId: string,
    connectionId: string | undefined,
    mode: 'auto' | 'manual',
    reason?: string,
  ): ReconnectSnapshot {
    const now = Date.now()
    const request: ReconnectRequest = {
      tunnelId,
      connectionId,
      mode,
      attempt: 0,
      reason,
      scheduledDelayMs: 0,
      nextRetryAt: now,
      createdAt: now,
    }
    this.queue.push(request)

    const snapshot: ReconnectSnapshot = {
      tunnelId,
      connectionId,
      state: 'queued',
      mode,
      policy: this.policy,
      attempt: 0,
      reconnectCount: 0,
      failedCount: 0,
      updatedAt: now,
    }
    this.snapshots.set(tunnelId, snapshot)
    return { ...snapshot }
  }

  private delayForAttempt(attempt: number): number | undefined {
    if (attempt <= 0 || attempt > this.policy.maxAttempts) {
      return undefined
    }

    switch (this.policy.kind) {
      case 'immediate':
        return 0
      case 'linear':
        return this.policy.delayMs * attempt
      case 'exponential_backoff':
        return Math.min(
          this.policy.baseDelayMs * this.policy.factor ** (attempt - 1),
          this.policy.maxDelayMs,
        )
      case 'fixed_interval':
        return this.policy.intervalMs
      case 'custom':
        return this.policy.delaysMs[attempt - 1]
    }
  }
}

/** Promise-based connection monitor service. */
export class ConnectionMonitorService {
  private readonly snapshots = new Map<string, ConnectionHealth>()
  private readonly pingStartedAt = new Map<string, number>()

  /** Register a connection for monitoring. */
  async register(tunnelId: string, connectionId: string): Promise<ConnectionHealth> {
    const snapshot: ConnectionHealth = {
      tunnelId,
      connectionId,
      alive: true,
      packetLoss: 0,
      averageRttMs: 0,
      healthScore: 100,
      status: 'healthy',
      connectionDurationMs: 0,
    }
    this.snapshots.set(connectionId, snapshot)
    return { ...snapshot }
  }

  /** Record ping observation. */
  async recordPing(connectionId: string): Promise<ConnectionHealth | undefined> {
    const snapshot = this.snapshots.get(connectionId)
    if (!snapshot) {
      return undefined
    }

    snapshot.lastPingAt = Date.now()
    this.pingStartedAt.set(connectionId, snapshot.lastPingAt)
    this.recompute(snapshot)
    return { ...snapshot }
  }

  /** Record pong observation and RTT. */
  async recordPong(connectionId: string, rttMs?: number): Promise<ConnectionHealth | undefined> {
    const snapshot = this.snapshots.get(connectionId)
    if (!snapshot) {
      return undefined
    }

    const startedAt = this.pingStartedAt.get(connectionId)
    const measuredRttMs = rttMs ?? (startedAt ? Math.max(0, Date.now() - startedAt) : 0)
    snapshot.alive = true
    snapshot.latencyMs = measuredRttMs
    snapshot.averageRttMs = snapshot.averageRttMs
      ? (snapshot.averageRttMs + measuredRttMs) / 2
      : measuredRttMs
    snapshot.lastPongAt = Date.now()
    this.recompute(snapshot)
    return { ...snapshot }
  }

  /** Mark connection as lost. */
  async markLost(connectionId: string): Promise<ConnectionHealth | undefined> {
    const snapshot = this.snapshots.get(connectionId)
    if (!snapshot) {
      return undefined
    }

    snapshot.alive = false
    this.recompute(snapshot)
    return { ...snapshot }
  }

  /** Mark connection as restored. */
  async markRestored(connectionId: string): Promise<ConnectionHealth | undefined> {
    const snapshot = this.snapshots.get(connectionId)
    if (!snapshot) {
      return undefined
    }

    snapshot.alive = true
    this.recompute(snapshot)
    return { ...snapshot }
  }

  /** Return the latest connection monitor snapshot. */
  async snapshot(connectionId: string): Promise<ConnectionHealth | undefined> {
    const snapshot = this.snapshots.get(connectionId)
    return snapshot ? { ...snapshot } : undefined
  }

  private recompute(snapshot: ConnectionHealth) {
    const latencyPenalty = Math.min(Math.floor((snapshot.latencyMs ?? 0) / 20), 40)
    const lossPenalty = Math.min(Math.round(snapshot.packetLoss * 50), 50)
    const alivePenalty = snapshot.alive ? 0 : 100
    snapshot.healthScore = Math.max(0, 100 - latencyPenalty - lossPenalty - alivePenalty)
    snapshot.status = statusFromScore(snapshot.alive ? snapshot.healthScore : 0)
  }
}

/** Promise-based session recovery service. */
export class SessionRecoveryService {
  private readonly contexts = new Map<string, RecoveryContext>()

  /** Capture recoverable runtime metadata. */
  async capture(context: RecoveryContext): Promise<void> {
    this.contexts.set(context.tunnelId, { ...context })
  }

  /** Recover runtime metadata for a tunnel. */
  async recover(tunnelId: string): Promise<RecoveryResult | undefined> {
    const startedAt = Date.now()
    const context = this.contexts.get(tunnelId)
    if (!context) {
      return undefined
    }

    return {
      tunnelId,
      recoveredSession: Boolean(context.sessionId),
      recoveredTunnel: true,
      recoveredStatistics: Boolean(context.statistics),
      recoveredContext: true,
      recoveredSubscription: context.subscriptions.length > 0,
      recoveryTimeMs: Math.max(1, Date.now() - startedAt),
      warnings: context.sessionId ? [] : ['session id missing; skipped session restore'],
    }
  }
}

/** Promise-based health aggregation service. */
export class HealthService {
  private readonly signals = new Map<HealthSignal['target'], HealthSignal>()

  /** Record a component health signal and return the aggregate report. */
  async recordSignal(signal: HealthSignal): Promise<HealthReport> {
    this.signals.set(signal.target, { ...signal })
    return this.report()
  }

  /** Build a connection health signal. */
  async checkConnection(alive: boolean, score: number): Promise<HealthSignal> {
    const signal: HealthSignal = {
      target: 'connection',
      status: alive ? statusFromScore(score) : 'offline',
      score: alive ? score : 0,
      checkedAt: Date.now(),
    }
    this.signals.set(signal.target, signal)
    return { ...signal }
  }

  /** Build a heartbeat health signal. */
  async checkHeartbeat(state: HeartbeatState): Promise<HealthSignal> {
    const signal = heartbeatSignal(state)
    this.signals.set(signal.target, signal)
    return { ...signal }
  }

  /** Return the aggregate health report. */
  async report(): Promise<HealthReport> {
    const components = Array.from(this.signals.values()).map((signal) => ({ ...signal }))
    if (components.length === 0) {
      return {
        status: 'warning',
        score: 80,
        checkedAt: Date.now(),
        components,
      }
    }

    const score = Math.round(
      components.reduce((sum, signal) => sum + signal.score, 0) / components.length,
    )
    const status = aggregateStatus(components, score)
    return {
      status,
      score,
      checkedAt: Date.now(),
      components,
    }
  }
}

/** Promise-based state synchronization service. */
export class StateSyncService {
  private readonly snapshots = new Map<string, StateSnapshot>()

  constructor(private readonly config: SyncConfig = defaultSyncConfig()) {}

  /** Prepare and record a state synchronization snapshot. */
  async synchronize<TPayload>(
    target: SyncTarget,
    payload: TPayload,
    tunnelId?: string,
  ): Promise<StateSyncResult> {
    if (!this.targetEnabled(target)) {
      throw new Error(`sync target disabled: ${target}`)
    }

    const key = this.key(target, tunnelId)
    const previous = this.snapshots.get(key)
    const version = (previous?.version ?? 0) + 1
    const synchronizedAt = Date.now()
    this.snapshots.set(key, {
      tunnelId,
      target,
      version,
      payload,
      synchronizedAt,
    })

    return {
      tunnelId,
      target,
      version,
      synchronized: true,
      synchronizedAt,
    }
  }

  /** Return the latest synchronized state snapshot. */
  async snapshot<TPayload>(
    target: SyncTarget,
    tunnelId?: string,
  ): Promise<StateSnapshot<TPayload> | undefined> {
    const snapshot = this.snapshots.get(this.key(target, tunnelId))
    return snapshot ? ({ ...snapshot } as StateSnapshot<TPayload>) : undefined
  }

  private key(target: SyncTarget, tunnelId?: string) {
    return `${tunnelId ?? 'global'}:${target}`
  }

  private targetEnabled(target: SyncTarget) {
    switch (target) {
      case 'tunnel_state':
        return this.config.includeTunnelState
      case 'project_state':
        return this.config.includeProjectState
      case 'server_state':
        return this.config.includeServerState
      case 'configuration':
        return this.config.includeConfiguration
      case 'statistics':
        return this.config.includeStatistics
      case 'log_cursor':
        return this.config.includeLogCursor
    }
  }
}

const statusFromScore = (score: number): HealthStatus => {
  if (score <= 0) {
    return 'offline'
  }

  if (score < 50) {
    return 'critical'
  }

  if (score < 80) {
    return 'warning'
  }

  return 'healthy'
}

const heartbeatSignal = (state: HeartbeatState): HealthSignal => {
  switch (state) {
    case 'running':
    case 'waiting_pong':
      return { target: 'heartbeat', status: 'healthy', score: 100, checkedAt: Date.now() }
    case 'retrying':
      return {
        target: 'heartbeat',
        status: 'warning',
        score: 80,
        message: 'heartbeat retrying',
        checkedAt: Date.now(),
      }
    case 'timeout':
      return {
        target: 'heartbeat',
        status: 'critical',
        score: 25,
        message: 'heartbeat timeout',
        checkedAt: Date.now(),
      }
    case 'idle':
      return {
        target: 'heartbeat',
        status: 'warning',
        score: 70,
        message: 'heartbeat idle',
        checkedAt: Date.now(),
      }
    case 'stopped':
      return {
        target: 'heartbeat',
        status: 'offline',
        score: 0,
        message: 'heartbeat stopped',
        checkedAt: Date.now(),
      }
  }
}

const aggregateStatus = (components: HealthSignal[], score: number): HealthStatus => {
  if (components.some((signal) => signal.status === 'offline')) {
    return 'offline'
  }

  if (components.some((signal) => signal.status === 'critical')) {
    return 'critical'
  }

  if (components.some((signal) => signal.status === 'warning')) {
    return 'warning'
  }

  return statusFromScore(score)
}
