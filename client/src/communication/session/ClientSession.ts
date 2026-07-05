import { createCommunicationId } from "../shared/id"
import type { ConnectionId, Session, SessionState } from "../types"

export class ClientSession implements Session {
  readonly id = createCommunicationId("sess")
  state: SessionState = "created"
  connectionId?: ConnectionId
  readonly createdAt = Date.now()
  updatedAt = this.createdAt
  attributes: Record<string, string> = {}

  attach(connectionId: ConnectionId) {
    this.connectionId = connectionId
    this.state = "active"
    this.updatedAt = Date.now()
  }

  close() {
    this.state = "closed"
    this.updatedAt = Date.now()
  }
}
