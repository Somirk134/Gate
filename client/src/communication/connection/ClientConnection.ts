import { createCommunicationId } from '../shared/id'
import type {
  Connection,
  ConnectionMetadata,
  ConnectionState,
  ConnectionStatistics,
} from '../types'

const createEmptyStatistics = (): ConnectionStatistics => ({
  connectedCount: 0,
  reconnectCount: 0,
  failedCount: 0,
  sendCount: 0,
  receiveCount: 0,
  averageLatencyMs: 0,
})

export class ClientConnection implements Connection {
  readonly id = createCommunicationId('conn')
  state: ConnectionState = 'created'
  statistics = createEmptyStatistics()
  readonly createdAt = Date.now()
  updatedAt = this.createdAt

  constructor(readonly metadata: ConnectionMetadata = {}) {}

  transition(state: ConnectionState) {
    this.state = state
    this.updatedAt = Date.now()

    if (state === 'connected') {
      this.statistics.connectedCount += 1
    }

    if (state === 'reconnecting') {
      this.statistics.reconnectCount += 1
    }

    if (state === 'failed') {
      this.statistics.failedCount += 1
    }
  }

  recordSend() {
    this.statistics.sendCount += 1
    this.updatedAt = Date.now()
  }

  recordReceive(latencyMs?: number) {
    this.statistics.receiveCount += 1

    if (latencyMs !== undefined) {
      this.statistics.averageLatencyMs = latencyMs
    }

    this.updatedAt = Date.now()
  }
}
