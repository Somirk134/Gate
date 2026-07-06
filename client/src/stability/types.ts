/** Heartbeat state exposed by the client stability API. */
export type HeartbeatState =
  'idle' | 'running' | 'waiting_pong' | 'timeout' | 'retrying' | 'stopped'

/** Unified health status emitted by stability services. */
export type HealthStatus = 'healthy' | 'warning' | 'critical' | 'offline'

/** Reconnect strategy policy. */
export type ReconnectPolicy =
  | { kind: 'immediate'; maxAttempts: number }
  | { kind: 'linear'; delayMs: number; maxAttempts: number }
  | {
      kind: 'exponential_backoff'
      baseDelayMs: number
      maxDelayMs: number
      factor: number
      maxAttempts: number
    }
  | { kind: 'fixed_interval'; intervalMs: number; maxAttempts: number }
  | { kind: 'custom'; delaysMs: number[]; maxAttempts: number }

/** Heartbeat configuration. */
export interface HeartbeatConfig {
  intervalMs: number
  timeoutMs: number
  retryCount: number
  retryDelayMs: number
  maxMissedHeartbeat: number
}

/** Heartbeat metrics shared with charts and health checks. */
export interface HeartbeatMetrics {
  pingCount: number
  pongCount: number
  timeoutCount: number
  retryCount: number
  heartbeatCount: number
  averageRttMs: number
  lastRttMs?: number
}

/** Heartbeat snapshot. */
export interface HeartbeatSnapshot {
  tunnelId: string
  state: HeartbeatState
  sequence: number
  missedHeartbeat: number
  retryAttempt: number
  lastPingAt?: number
  lastPongAt?: number
  lastTimeoutAt?: number
  metrics: HeartbeatMetrics
}

/** Connection health snapshot. */
export interface ConnectionHealth {
  tunnelId: string
  connectionId: string
  latencyMs?: number
  alive: boolean
  packetLoss: number
  lastPingAt?: number
  lastPongAt?: number
  averageRttMs: number
  healthScore: number
  status: HealthStatus
  connectionDurationMs: number
}

/** Reconnect lifecycle state. */
export type ReconnectState =
  'idle' | 'queued' | 'scheduling' | 'reconnecting' | 'succeeded' | 'failed' | 'cancelled'

/** Reconnect work item produced by the scheduler. */
export interface ReconnectRequest {
  tunnelId: string
  connectionId?: string
  mode: 'auto' | 'manual'
  attempt: number
  reason?: string
  scheduledDelayMs: number
  nextRetryAt: number
  createdAt: number
}

/** Reconnect snapshot. */
export interface ReconnectSnapshot {
  tunnelId: string
  connectionId?: string
  state: ReconnectState
  mode: 'auto' | 'manual'
  policy: ReconnectPolicy
  attempt: number
  reconnectCount: number
  failedCount: number
  lastError?: string
  nextRetryAt?: number
  updatedAt: number
}

/** Recoverable subscription cursor. */
export interface SubscriptionSnapshot {
  name: string
  cursor?: string
}

/** Recoverable runtime context. Business data is intentionally excluded. */
export interface RecoveryContext {
  tunnelId: string
  sessionId?: string
  connectionId?: string
  statistics?: Partial<HeartbeatMetrics>
  subscriptions: SubscriptionSnapshot[]
  attributes: Record<string, string>
  capturedAt: number
}

/** Recovery result returned by SessionRecoveryManager-compatible clients. */
export interface RecoveryResult {
  tunnelId: string
  recoveredSession: boolean
  recoveredTunnel: boolean
  recoveredStatistics: boolean
  recoveredContext: boolean
  recoveredSubscription: boolean
  recoveryTimeMs: number
  warnings: string[]
}

/** Health component signal. */
export interface HealthSignal {
  target: 'connection' | 'heartbeat' | 'authentication' | 'runtime' | 'tunnel' | 'server'
  status: HealthStatus
  score: number
  message?: string
  checkedAt: number
}

/** Unified health report. */
export interface HealthReport {
  status: HealthStatus
  score: number
  checkedAt: number
  components: HealthSignal[]
}

/** State sync target. */
export type SyncTarget =
  'tunnel_state' | 'project_state' | 'server_state' | 'configuration' | 'statistics' | 'log_cursor'

/** State snapshot prepared for synchronization. */
export interface StateSnapshot<TPayload = unknown> {
  tunnelId?: string
  target: SyncTarget
  version: number
  payload: TPayload
  synchronizedAt: number
}

/** State sync result. */
export interface StateSyncResult {
  tunnelId?: string
  target: SyncTarget
  version: number
  synchronized: boolean
  synchronizedAt: number
}

/** State sync configuration. */
export interface SyncConfig {
  intervalMs: number
  batchSize: number
  includeTunnelState: boolean
  includeProjectState: boolean
  includeServerState: boolean
  includeConfiguration: boolean
  includeStatistics: boolean
  includeLogCursor: boolean
}
