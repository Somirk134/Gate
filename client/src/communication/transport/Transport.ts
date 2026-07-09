import type { Message, Transport, TransportEndpoint } from '../types'

export class ClientTransport {
  constructor(private readonly inner: Transport) {}

  get name() {
    return this.inner.name
  }

  get state() {
    return this.inner.state
  }

  connect(endpoint: TransportEndpoint) {
    return this.inner.connect(endpoint)
  }

  disconnect() {
    return this.inner.disconnect()
  }

  send(message: Message) {
    return this.inner.send(message)
  }

  receive() {
    return this.inner.receive()
  }

  reconnect() {
    return this.inner.reconnect()
  }
}
