import { ClientConnection } from "./connection/ClientConnection"
import { ClientDispatcher } from "./dispatcher/ClientDispatcher"
import { ClientEventManager } from "./event/ClientEventManager"
import { ClientRequestManager } from "./request/ClientRequestManager"
import { ClientSession } from "./session/ClientSession"
import { createCommunicationId } from "./shared/id"
import { mergeTimeoutConfig } from "./shared/timeout"
import { ClientTransport } from "./transport/Transport"
import type {
  Command,
  Connection,
  EventHandler,
  Message,
  RequestOptions,
  Response,
  SubscribeOptions,
  TimeoutConfig,
  Transport,
  TransportEndpoint,
  Unsubscribe,
} from "./types"

export interface CommunicationServiceOptions {
  transport: Transport
  timeoutConfig?: Partial<TimeoutConfig>
}

/** Promise-based client API used by Vue/Tauri application services. */
export class CommunicationService {
  private readonly transport: ClientTransport
  private readonly requests = new ClientRequestManager()
  private readonly events = new ClientEventManager()
  private readonly dispatcher = new ClientDispatcher(this.requests, this.events)
  private readonly timeoutConfig: TimeoutConfig
  private readonly session = new ClientSession()
  private connection?: ClientConnection

  constructor(options: CommunicationServiceOptions) {
    this.transport = new ClientTransport(options.transport)
    this.timeoutConfig = mergeTimeoutConfig(options.timeoutConfig)
  }

  async connect(endpoint: TransportEndpoint): Promise<Connection> {
    this.connection = new ClientConnection({
      endpoint: endpoint.url ?? endpoint.value ?? endpoint.host,
      transport: endpoint.kind,
      protocolVersion: "v1",
    })
    this.connection.transition("connecting")

    await this.transport.connect(endpoint)

    this.connection.transition("connected")
    this.session.attach(this.connection.id)
    return this.connection
  }

  async disconnect() {
    await this.transport.disconnect()
    this.connection?.transition("disconnected")
    this.session.close()
    this.requests.clear()
  }

  async request<TResponse = unknown, TBody = unknown>(
    command: Command,
    body: TBody,
    options: RequestOptions = {},
  ): Promise<Response<TResponse>> {
    const request = this.requests.createRequest(command, body, {
      timeoutMs: options.timeoutMs ?? this.timeoutConfig.requestTimeoutMs,
      retryPolicy: options.retryPolicy,
      signal: options.signal,
    })
    const response = this.requests.register<TResponse>(request)

    try {
      await this.transport.send(request.message)
      this.connection?.recordSend()
    } catch (error) {
      this.requests.cancel(request.id)
      throw error
    }

    options.signal?.addEventListener(
      "abort",
      () => this.requests.cancel(request.id),
      { once: true },
    )

    return response
  }

  async notify<TBody = unknown>(command: Command, body: TBody) {
    await this.transport.send(this.createMessage("notification", command, body))
    this.connection?.recordSend()
  }

  async subscribe<TPayload>(
    name: string,
    handler: EventHandler<TPayload>,
    options?: SubscribeOptions,
  ): Promise<Unsubscribe> {
    return this.events.subscribe(name, handler, options)
  }

  async unsubscribe(name: string, subscriptionId: string) {
    return this.events.unsubscribe(name, subscriptionId)
  }

  async handleIncoming(message: Message) {
    await this.dispatcher.dispatch(message)
    this.connection?.recordReceive(Math.max(0, Date.now() - message.header.timestamp))
  }

  currentConnection() {
    return this.connection
  }

  currentSession() {
    return this.session
  }

  private createMessage<TBody>(
    messageType: Message<TBody>["header"]["messageType"],
    command: Command,
    body: TBody,
  ): Message<TBody> {
    return {
      header: {
        protocolVersion: "v1",
        messageType,
        command,
        requestId: createCommunicationId("req"),
        traceId: createCommunicationId("trace"),
        timestamp: Date.now(),
        sequence: 0,
      },
      body,
      metadata: {},
    }
  }
}
