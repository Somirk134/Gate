import type {
  Message,
  Transport,
  TransportCapabilities,
  TransportEndpoint,
  TransportKind,
  TransportState,
} from "../types"

export class MockTransport implements Transport {
  readonly name = "mock"
  readonly kind: TransportKind = "mock"
  readonly capabilities: TransportCapabilities = {
    kind: "mock",
    supportsReconnect: true,
    supportsPriorityQueue: true,
    supportsBinary: true,
  }

  private incoming: Message[] = []
  private outgoing: Message[] = []
  private endpoint?: TransportEndpoint
  private reconnectCountValue = 0
  state: TransportState = "created"

  async connect(endpoint: TransportEndpoint) {
    this.state = "connecting"
    this.endpoint = endpoint
    this.state = "connected"
  }

  async disconnect() {
    this.state = "disconnected"
  }

  async send(message: Message) {
    this.outgoing.push(message)
  }

  async receive() {
    return this.incoming.shift()
  }

  async reconnect() {
    this.state = "reconnecting"
    this.reconnectCountValue += 1
    this.state = "connected"
  }

  pushIncoming(message: Message) {
    this.incoming.push(message)
  }

  popOutgoing() {
    return this.outgoing.shift()
  }

  reconnectCount() {
    return this.reconnectCountValue
  }

  currentEndpoint() {
    return this.endpoint
  }
}
