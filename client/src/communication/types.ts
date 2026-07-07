export type JsonValue =
  | null
  | boolean
  | number
  | string
  | JsonValue[]
  | { [key: string]: JsonValue }

export type Command = string
export type RequestId = string
export type ConnectionId = string
export type SessionId = string
export type EventName = string

export type MessageType =
  | "request"
  | "response"
  | "event"
  | "heartbeat"
  | "notification"
  | "broadcast"
  | "error"
  | "ack"
  | "plugin"

export type ConnectionState =
  | "created"
  | "connecting"
  | "connected"
  | "authenticated"
  | "running"
  | "reconnecting"
  | "disconnected"
  | "closed"
  | "failed"

export type ClientState = ConnectionState

export type ServerState =
  | "created"
  | "starting"
  | "running"
  | "draining"
  | "stopped"
  | "failed"

export type TransportState =
  | "created"
  | "connecting"
  | "connected"
  | "running"
  | "reconnecting"
  | "disconnecting"
  | "disconnected"
  | "closed"
  | "failed"

export type SessionState = "created" | "active" | "suspended" | "closed"

export type TransportKind = "tcp" | "websocket" | "quic" | "custom"

export interface TransportEndpoint {
  kind: TransportKind
  host?: string
  port?: number
  url?: string
  value?: string
}

export interface TransportCapabilities {
  kind: TransportKind
  supportsReconnect: boolean
  supportsPriorityQueue: boolean
  supportsBinary: boolean
}

export interface MessageMetadata {
  clientVersion?: string
  platform?: string
  os?: string
  language?: string
  architecture?: string
  extra?: Record<string, string>
}

export interface MessageHeader {
  protocolVersion: string
  messageType: MessageType
  command: Command
  requestId: RequestId
  traceId: string
  timestamp: number
  sequence: number
}

/** Protocol-level message envelope used by the client communication layer. */
export interface Message<TBody = unknown> {
  header: MessageHeader
  body: TBody
  metadata: MessageMetadata
}

export type RetryPolicyConfig =
  | { kind: "none" }
  | {
      kind: "linear"
      initialDelayMs: number
      maxDelayMs: number
      maxAttempts: number
    }
  | {
      kind: "exponential"
      baseDelayMs: number
      maxDelayMs: number
      factor: number
      maxAttempts: number
    }
  | { kind: "custom"; delaysMs: number[] }

export interface RequestOptions {
  timeoutMs?: number
  retryPolicy?: RetryPolicyConfig
  signal?: AbortSignal
}

/** Request tracked by ClientRequestManager until its response arrives. */
export interface Request<TBody = unknown> {
  id: RequestId
  command: Command
  message: Message<TBody>
  timeoutMs: number
  createdAt: number
  retryPolicy?: RetryPolicyConfig
}

/** Response resolved by request id. */
export interface Response<TBody = unknown> {
  requestId: RequestId
  command: Command
  message: Message<TBody>
  body: TBody
  receivedAt: number
  latencyMs?: number
}

/** Publish/subscribe event envelope. */
export interface Event<TPayload = unknown> {
  id: string
  name: EventName
  payload: TPayload
  source?: string
  priority: number
  timestamp: number
}

export type EventHandler<TPayload = unknown> = (
  event: Event<TPayload>,
) => void | Promise<void>

export interface SubscribeOptions {
  once?: boolean
  priority?: number
  signal?: AbortSignal
}

export type Unsubscribe = () => void

export interface ConnectionMetadata {
  endpoint?: string
  transport?: string
  clientId?: string
  serverId?: string
  protocolVersion?: string
  tags?: Record<string, string>
}

export interface ConnectionStatistics {
  connectedCount: number
  reconnectCount: number
  failedCount: number
  sendCount: number
  receiveCount: number
  averageLatencyMs: number
}

/** Client/server connection contract shared by UI stores and services. */
export interface Connection {
  id: ConnectionId
  state: ConnectionState
  metadata: ConnectionMetadata
  statistics: ConnectionStatistics
  createdAt: number
  updatedAt: number
}

/** Session context reserved for future persisted session stores. */
export interface Session {
  id: SessionId
  state: SessionState
  connectionId?: ConnectionId
  createdAt: number
  updatedAt: number
  attributes: Record<string, string>
}

/** Async transport boundary. Real TCP/WebSocket/QUIC adapters plug in here. */
export interface Transport {
  readonly name: string
  readonly kind: TransportKind
  readonly state: TransportState
  readonly capabilities: TransportCapabilities
  connect(endpoint: TransportEndpoint): Promise<void>
  disconnect(): Promise<void>
  send(message: Message): Promise<void>
  receive(): Promise<Message | undefined>
  reconnect(): Promise<void>
}

export interface TimeoutConfig {
  requestTimeoutMs: number
  heartbeatTimeoutMs: number
  connectionTimeoutMs: number
  readTimeoutMs: number
  writeTimeoutMs: number
}
