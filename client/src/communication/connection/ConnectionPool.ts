import type { Connection, ConnectionId } from "../types"

/** Reserved client-side pool for future multi-connection workflows. */
export class ConnectionPool {
  private readonly connections = new Map<ConnectionId, Connection>()

  insert(connection: Connection) {
    this.connections.set(connection.id, connection)
    return connection
  }

  get(connectionId: ConnectionId) {
    return this.connections.get(connectionId)
  }

  remove(connectionId: ConnectionId) {
    const connection = this.connections.get(connectionId)
    this.connections.delete(connectionId)
    return connection
  }

  clear() {
    this.connections.clear()
  }

  size() {
    return this.connections.size
  }
}
